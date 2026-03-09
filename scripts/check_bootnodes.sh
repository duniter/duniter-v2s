#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLIENT_SPEC_FILE="${1:-${ROOT_DIR}/node/specs/g1_client-specs.yaml}"
RAW_SPEC_FILE="${2:-${ROOT_DIR}/node/specs/g1-raw.json}"
BUILD_PROFILE="${DUNITER_BUILD_PROFILE:-debug}"
BOOTNODE_TIMEOUT_SECONDS="${BOOTNODE_TIMEOUT_SECONDS:-60}"
RESULTS_DIR="${BOOTNODE_RESULTS_DIR:-${ROOT_DIR}/target/bootnode-check}"
ISOLATED_RAW_SPEC_FILE=""

usage() {
  cat <<'EOF'
Usage: scripts/check_bootnodes.sh [client-spec-file] [raw-spec-file]

Checks every bootnode listed in the client spec and reports:
- unreachable bootnodes
- malformed bootnode entries
- peer ID mismatches

Environment variables:
- DUNITER_BINARY: override the node binary path
- DUNITER_BUILD_PROFILE: target profile to use when DUNITER_BINARY is unset (debug or release)
- BOOTNODE_TIMEOUT_SECONDS: per-bootnode timeout in seconds (default: 60)
- BOOTNODE_RESULTS_DIR: directory used for logs and per-node results

Defaults:
- client spec: node/specs/g1_client-specs.yaml
- raw spec: node/specs/g1-raw.json
EOF
}

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

case "$BUILD_PROFILE" in
  debug|release)
    ;;
  *)
    echo "Invalid DUNITER_BUILD_PROFILE='$BUILD_PROFILE' (expected 'debug' or 'release')." >&2
    exit 1
    ;;
esac

if [ -n "${DUNITER_BINARY:-}" ]; then
  DUNITER_BINARY="${DUNITER_BINARY}"
else
  DUNITER_BINARY="${ROOT_DIR}/target/${BUILD_PROFILE}/duniter"
  if [ "$BUILD_PROFILE" = "release" ] && [ ! -x "$DUNITER_BINARY" ] && [ -x "${ROOT_DIR}/target/debug/duniter" ]; then
    DUNITER_BINARY="${ROOT_DIR}/target/debug/duniter"
  fi
fi

if [ ! -f "$CLIENT_SPEC_FILE" ]; then
  echo "Client spec file not found: $CLIENT_SPEC_FILE" >&2
  exit 1
fi

if [ ! -f "$RAW_SPEC_FILE" ]; then
  echo "Raw spec file not found: $RAW_SPEC_FILE" >&2
  exit 1
fi

if [ ! -x "$DUNITER_BINARY" ]; then
  echo "Duniter binary not found or not executable: $DUNITER_BINARY" >&2
  echo "Build it first, for example:" >&2
  echo "  cargo build --release --locked --package duniter --bin duniter --no-default-features --features g1" >&2
  exit 1
fi

if ! [[ "$BOOTNODE_TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || [ "$BOOTNODE_TIMEOUT_SECONDS" -le 0 ]; then
  echo "BOOTNODE_TIMEOUT_SECONDS must be a positive integer." >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required to prepare the raw chain spec." >&2
  exit 1
fi

mkdir -p "$RESULTS_DIR"
rm -f "$RESULTS_DIR"/result_*.txt "$RESULTS_DIR"/node_*.log

cleanup() {
  if [ -n "${ISOLATED_RAW_SPEC_FILE:-}" ] && [ -f "$ISOLATED_RAW_SPEC_FILE" ]; then
    rm -f "$ISOLATED_RAW_SPEC_FILE"
  fi
}
trap cleanup EXIT INT TERM

RAW_SPEC_BASENAME="$(basename "$RAW_SPEC_FILE" .json)"
ISOLATED_RAW_SPEC_FILE="$(mktemp "${RESULTS_DIR}/${RAW_SPEC_BASENAME}.isolated.XXXXXX.json")"
jq '.bootNodes = []' "$RAW_SPEC_FILE" > "$ISOLATED_RAW_SPEC_FILE"

extract_bootnodes() {
  awk '
    $1 == "bootNodes:" { in_bootnodes = 1; next }
    in_bootnodes && $0 ~ /^[^[:space:]][^:]*:/ { exit }
    in_bootnodes && $1 == "-" {
      sub(/^[[:space:]]*-[[:space:]]*/, "", $0)
      sub(/^"/, "", $0)
      sub(/"$/, "", $0)
      print $0
    }
  ' "$CLIENT_SPEC_FILE"
}

extract_expected_peer_id() {
  local bootnode="$1"
  printf '%s\n' "$bootnode" | sed -n 's#^.*/p2p/\([^/][^[:space:]]*\)$#\1#p'
}

extract_actual_peer_id() {
  local log_file="$1"
  sed -n 's/.*provided a different peer ID `\([^`][^`]*\)`.*/\1/p' "$log_file" | awk 'NR == 1 { print; exit }'
}

log_indicates_success() {
  local log_file="$1"
  grep -qE '\([1-9][0-9]* peers?\)' "$log_file" 2>/dev/null
}

write_result() {
  local result_file="$1"
  local status="$2"
  local bootnode="$3"
  local expected_peer_id="$4"
  local actual_peer_id="$5"
  local reason="$6"

  {
    printf 'status=%s\n' "$status"
    printf 'bootnode=%s\n' "$bootnode"
    printf 'expected_peer_id=%s\n' "$expected_peer_id"
    printf 'actual_peer_id=%s\n' "$actual_peer_id"
    printf 'reason=%s\n' "$reason"
  } > "$result_file"
}

stop_process() {
  local pid="$1"
  local grace_seconds="${2:-5}"

  if ! kill -0 "$pid" 2>/dev/null; then
    return
  fi

  kill -TERM "$pid" 2>/dev/null || true

  for _ in $(seq 1 "$grace_seconds"); do
    if ! kill -0 "$pid" 2>/dev/null; then
      return
    fi
    sleep 1
  done

  kill -KILL "$pid" 2>/dev/null || true
}

check_bootnode() {
  local bootnode="$1"
  local index="$2"
  local base_port=$((30333 + index * 20))
  local log_file="${RESULTS_DIR}/node_${index}.log"
  local result_file="${RESULTS_DIR}/result_${index}.txt"
  local expected_peer_id
  local actual_peer_id=""
  local status="TIMEOUT"
  local reason="Could not establish a peer connection within ${BOOTNODE_TIMEOUT_SECONDS}s."
  local node_pid=""

  expected_peer_id="$(extract_expected_peer_id "$bootnode")"
  if [ -z "$expected_peer_id" ] || [[ "$bootnode" != /* ]]; then
    write_result "$result_file" "INVALID_MULTIADDR" "$bootnode" "$expected_peer_id" "" \
      "Bootnode entry must be a libp2p multiaddr ending with /p2p/<peer-id>."
    return
  fi

  "$DUNITER_BINARY" \
    --chain "$ISOLATED_RAW_SPEC_FILE" \
    --reserved-only \
    --reserved-nodes "$bootnode" \
    --tmp \
    --port "$base_port" \
    --rpc-port 0 \
    --no-mdns \
    --no-prometheus \
    > "$log_file" 2>&1 &
  node_pid=$!

  for _ in $(seq 1 "$BOOTNODE_TIMEOUT_SECONDS"); do
    sleep 1

    if grep -q "provided a different peer ID" "$log_file" 2>/dev/null; then
      actual_peer_id="$(extract_actual_peer_id "$log_file")"
      status="PEER_ID_MISMATCH"
      reason="Remote node answered with a different peer ID than the one declared in the chain spec."
      break
    fi

    if log_indicates_success "$log_file"; then
      status="SUCCESS"
      reason="Reserved connection established successfully."
      break
    fi

    if ! kill -0 "$node_pid" 2>/dev/null; then
      status="NODE_EXITED"
      reason="The local duniter process exited before a connection was established."
      break
    fi
  done

  stop_process "$node_pid"
  wait "$node_pid" 2>/dev/null || true

  write_result "$result_file" "$status" "$bootnode" "$expected_peer_id" "$actual_peer_id" "$reason"
}

BOOTNODES=()
while IFS= read -r bootnode; do
  [ -n "$bootnode" ] || continue
  BOOTNODES+=("$bootnode")
done < <(extract_bootnodes)

if [ "${#BOOTNODES[@]}" -eq 0 ]; then
  echo "No bootnodes found in $CLIENT_SPEC_FILE" >&2
  exit 1
fi

echo "Checking ${#BOOTNODES[@]} bootnodes from $CLIENT_SPEC_FILE"
echo "Using raw chain spec: $RAW_SPEC_FILE"
echo "Using binary: $DUNITER_BINARY"
echo

index=0
for bootnode in "${BOOTNODES[@]}"; do
  check_bootnode "$bootnode" "$index" &
  index=$((index + 1))
  sleep 0.2
done
wait

success_count=0
invalid_count=0

for result_file in "$RESULTS_DIR"/result_*.txt; do
  [ -f "$result_file" ] || continue

  status="$(awk -F= '/^status=/{print substr($0, index($0, "=") + 1)}' "$result_file")"
  bootnode="$(awk -F= '/^bootnode=/{print substr($0, index($0, "=") + 1)}' "$result_file")"
  expected_peer_id="$(awk -F= '/^expected_peer_id=/{print substr($0, index($0, "=") + 1)}' "$result_file")"
  actual_peer_id="$(awk -F= '/^actual_peer_id=/{print substr($0, index($0, "=") + 1)}' "$result_file")"
  reason="$(awk -F= '/^reason=/{print substr($0, index($0, "=") + 1)}' "$result_file")"

  case "$status" in
    SUCCESS)
      echo "OK: $bootnode"
      success_count=$((success_count + 1))
      ;;
    PEER_ID_MISMATCH)
      echo "INVALID: $bootnode"
      echo "  reason: $reason"
      echo "  expected peer ID: $expected_peer_id"
      echo "  actual peer ID: ${actual_peer_id:-<unknown>}"
      invalid_count=$((invalid_count + 1))
      ;;
    *)
      echo "INVALID: $bootnode"
      echo "  reason: $reason"
      [ -n "$expected_peer_id" ] && echo "  expected peer ID: $expected_peer_id"
      invalid_count=$((invalid_count + 1))
      ;;
  esac
done

echo
echo "Summary: ${success_count}/${#BOOTNODES[@]} bootnodes valid"

if [ "$invalid_count" -ne 0 ]; then
  echo "Logs and detailed results are available in $RESULTS_DIR" >&2
  exit 1
fi
