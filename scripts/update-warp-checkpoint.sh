#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/update-warp-checkpoint.sh <network>

Fetch a finalized head from one rpc endpoint, target block height `finalized - 10`,
then compare returned hashes and headers for that exact height across rpc endpoints.
Unusable endpoints are ignored and reported.

When the returned hashes/headers are inconsistent, the script computes the majority
hash and asks for interactive confirmation before writing it.

Arguments:
  network    One of g1, gdev, or gtest.

Output:
  Writes the validated header to node/specs/<network>-checkpoint.json.
EOF
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  usage
  exit 0
fi

if [ $# -ne 1 ]; then
  echo "Missing network argument (g1, gdev, or gtest)." >&2
  usage
  exit 1
fi

NETWORK="$1"
case "$NETWORK" in
  g1|gdev|gtest)
    ;;
  *)
    echo "Invalid network '$NETWORK' (expected g1, gdev, or gtest)." >&2
    exit 1
    ;;
esac

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required." >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required." >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NETWORK_SPEC_URL="https://git.duniter.org/nodes/networks/-/raw/master/${NETWORK}.json"
SPEC_FILE="${ROOT_DIR}/node/specs/${NETWORK}-checkpoint.json"
FIELD_SEPARATOR=$'\034'

extract_rpc_endpoints() {
  jq -r '
    .rpc // empty
    | if type == "array" then .[] else . end
    | if type == "string" then
        .
      elif type == "object" then
        .url // empty
      else
        empty
      end
    | select(. != null and . != "")
  '
}

network_json="$(curl -fsSL "$NETWORK_SPEC_URL")"
if [ -z "$network_json" ]; then
  echo "Failed to download network spec from ${NETWORK_SPEC_URL}" >&2
  exit 1
fi

RPC_ENDPOINTS=()
while IFS= read -r endpoint; do
  if [ -n "$endpoint" ]; then
    RPC_ENDPOINTS+=("$endpoint")
  fi
done < <(printf '%s\n' "$network_json" | extract_rpc_endpoints)

if [ "${#RPC_ENDPOINTS[@]}" -eq 0 ]; then
  echo "No rpc endpoints found in ${NETWORK_SPEC_URL}" >&2
  exit 1
fi

declare -a FAILED_ENDPOINTS=()
declare -a FAILED_REASONS=()
declare -a USABLE_ENDPOINTS=()
declare -a USABLE_HASHES=()
declare -a USABLE_HEADERS=()

record_failed_endpoint() {
  local endpoint="$1"
  local reason="$2"

  FAILED_ENDPOINTS+=("$endpoint")
  FAILED_REASONS+=("$reason")
}

rpc_call() {
  local endpoint="$1"
  local method="$2"
  local params_json="$3"
  local response
  local payload
  local call_endpoint="$endpoint"
  local using_http_fallback="false"

  payload="$(jq -nc --arg method "$method" --argjson params "$params_json" \
    '{jsonrpc:"2.0",id:1,method:$method,params:$params}')"

  case "$endpoint" in
    ws://*)
      call_endpoint="http://${endpoint#ws://}"
      using_http_fallback="true"
      ;;
    wss://*)
      call_endpoint="https://${endpoint#wss://}"
      using_http_fallback="true"
      ;;
  esac

  if ! response="$(curl -fsSL -m 30 -H 'Content-Type: application/json' --data "$payload" "$call_endpoint")"; then
    if [ "$using_http_fallback" = "true" ]; then
      echo "RPC call failed for $endpoint (ws/wss unsupported by curl, tried $call_endpoint): $method" >&2
    else
      echo "RPC call failed: $method on $endpoint" >&2
    fi
    return 1
  fi

  if ! printf '%s' "$response" | jq -e '.jsonrpc == "2.0"' >/dev/null; then
    echo "Invalid JSON-RPC response from $endpoint: $response" >&2
    return 1
  fi

  if printf '%s' "$response" | jq -e '.error' >/dev/null; then
    echo "RPC error from $endpoint for $method: $(printf '%s' "$response" | jq -r '.error.message // .error')" >&2
    return 1
  fi

  printf '%s' "$response"
}

query_finalized_hash() {
  local endpoint="$1"
  local response
  local block_hash

  response="$(rpc_call "$endpoint" "chain_getFinalizedHead" "[]")"
  block_hash="$(printf '%s' "$response" | jq -r '.result // empty')"
  if [ -z "$block_hash" ] || [ "$block_hash" = "null" ]; then
    echo "No finalized head returned by $endpoint." >&2
    return 1
  fi

  printf '%s\n' "$block_hash"
}

query_header_by_hash() {
  local endpoint="$1"
  local block_hash="$2"
  local response
  local header

  response="$(rpc_call "$endpoint" "chain_getHeader" "$(jq -nc --arg hash "$block_hash" '[ $hash ]')")"
  header="$(printf '%s' "$response" | jq -c '.result // empty')"
  if [ -z "$header" ] || [ "$header" = "null" ]; then
    echo "No header returned by $endpoint for block ${block_hash}." >&2
    return 1
  fi

  printf '%s\n' "$header" | jq -S -c .
}

query_block_hash_by_number() {
  local endpoint="$1"
  local block_number_hex="$2"
  local response
  local block_hash

  response="$(rpc_call "$endpoint" "chain_getBlockHash" "$(jq -nc --arg number "$block_number_hex" '[ $number ]')")"
  block_hash="$(printf '%s' "$response" | jq -r '.result // empty')"
  if [ -z "$block_hash" ] || [ "$block_hash" = "null" ]; then
    echo "No block hash returned by $endpoint for block number ${block_number_hex}." >&2
    return 1
  fi

  printf '%s\n' "$block_hash"
}

hex_to_dec() {
  local hex_value="${1#0x}"

  if [ -z "$hex_value" ]; then
    return 1
  fi

  printf '%s\n' "$((16#$hex_value))"
}

dec_to_hex() {
  local dec_value="$1"
  printf '0x%x\n' "$dec_value"
}

declare -a REFERENCE_FAILED_ENDPOINTS=()
declare -a REFERENCE_FAILED_REASONS=()

record_reference_failure() {
  local endpoint="$1"
  local reason="$2"

  REFERENCE_FAILED_ENDPOINTS+=("$endpoint")
  REFERENCE_FAILED_REASONS+=("$reason")
}

REFERENCE_ENDPOINT=""
REFERENCE_FINALIZED_HASH=""
REFERENCE_FINALIZED_HEADER=""
REFERENCE_FINALIZED_HEIGHT_DEC=""
REFERENCE_TARGET_HEIGHT_DEC=""
REFERENCE_TARGET_HEIGHT_HEX=""
REFERENCE_HASH=""
REFERENCE_HEADER=""

for RPC_ENDPOINT in "${RPC_ENDPOINTS[@]}"; do
  FINALIZED_HASH="$(query_finalized_hash "$RPC_ENDPOINT" || true)"
  if [ -z "$FINALIZED_HASH" ]; then
    record_reference_failure "$RPC_ENDPOINT" "chain_getFinalizedHead failed"
    continue
  fi

  FINALIZED_HEADER="$(query_header_by_hash "$RPC_ENDPOINT" "$FINALIZED_HASH" || true)"
  if [ -z "$FINALIZED_HEADER" ]; then
    record_reference_failure "$RPC_ENDPOINT" "chain_getHeader(${FINALIZED_HASH}) failed"
    continue
  fi

  FINALIZED_NUMBER_HEX="$(printf '%s' "$FINALIZED_HEADER" | jq -r '.number // empty')"
  if [ -z "$FINALIZED_NUMBER_HEX" ] || [ "$FINALIZED_NUMBER_HEX" = "null" ]; then
    record_reference_failure "$RPC_ENDPOINT" "missing block number in finalized header"
    continue
  fi

  FINALIZED_NUMBER_DEC="$(hex_to_dec "$FINALIZED_NUMBER_HEX" || true)"
  if [ -z "$FINALIZED_NUMBER_DEC" ]; then
    record_reference_failure "$RPC_ENDPOINT" "invalid finalized block number ${FINALIZED_NUMBER_HEX}"
    continue
  fi

  if [ "$FINALIZED_NUMBER_DEC" -lt 10 ]; then
    TARGET_HEIGHT_DEC=0
  else
    TARGET_HEIGHT_DEC=$((FINALIZED_NUMBER_DEC - 10))
  fi
  TARGET_HEIGHT_HEX="$(dec_to_hex "$TARGET_HEIGHT_DEC")"

  TARGET_HASH="$(query_block_hash_by_number "$RPC_ENDPOINT" "$TARGET_HEIGHT_HEX" || true)"
  if [ -z "$TARGET_HASH" ]; then
    record_reference_failure "$RPC_ENDPOINT" "chain_getBlockHash(${TARGET_HEIGHT_HEX}) failed"
    continue
  fi

  TARGET_HEADER="$(query_header_by_hash "$RPC_ENDPOINT" "$TARGET_HASH" || true)"
  if [ -z "$TARGET_HEADER" ]; then
    record_reference_failure "$RPC_ENDPOINT" "chain_getHeader(${TARGET_HASH}) failed"
    continue
  fi

  REFERENCE_ENDPOINT="$RPC_ENDPOINT"
  REFERENCE_FINALIZED_HASH="$FINALIZED_HASH"
  REFERENCE_FINALIZED_HEADER="$FINALIZED_HEADER"
  REFERENCE_FINALIZED_HEIGHT_DEC="$FINALIZED_NUMBER_DEC"
  REFERENCE_TARGET_HEIGHT_DEC="$TARGET_HEIGHT_DEC"
  REFERENCE_TARGET_HEIGHT_HEX="$TARGET_HEIGHT_HEX"
  REFERENCE_HASH="$TARGET_HASH"
  REFERENCE_HEADER="$TARGET_HEADER"
  break
done

if [ -z "$REFERENCE_ENDPOINT" ]; then
  echo "No RPC endpoint returned a usable finalized header and checkpoint target for ${NETWORK}." >&2
  if [ "${#REFERENCE_FAILED_ENDPOINTS[@]}" -gt 0 ]; then
    echo "Ignored RPC endpoints (errors):"
    for i in "${!REFERENCE_FAILED_ENDPOINTS[@]}"; do
      echo "  - ${REFERENCE_FAILED_ENDPOINTS[$i]}: ${REFERENCE_FAILED_REASONS[$i]}"
    done
  fi
  exit 1
fi

echo "Reference endpoint: ${REFERENCE_ENDPOINT}"
echo "Reference finalized head: ${REFERENCE_FINALIZED_HASH}"
echo "Reference finalized height: ${REFERENCE_FINALIZED_HEIGHT_DEC}"
echo "Target checkpoint height: ${REFERENCE_TARGET_HEIGHT_DEC} (${REFERENCE_TARGET_HEIGHT_HEX})"

for RPC_ENDPOINT in "${RPC_ENDPOINTS[@]}"; do
  HASH="$(query_block_hash_by_number "$RPC_ENDPOINT" "$REFERENCE_TARGET_HEIGHT_HEX" || true)"
  if [ -z "$HASH" ]; then
    record_failed_endpoint "$RPC_ENDPOINT" "chain_getBlockHash(${REFERENCE_TARGET_HEIGHT_HEX}) failed"
    continue
  fi

  HEADER="$(query_header_by_hash "$RPC_ENDPOINT" "$HASH" || true)"
  if [ -z "$HEADER" ]; then
    record_failed_endpoint "$RPC_ENDPOINT" "chain_getHeader(${HASH}) failed"
    continue
  fi

  USABLE_ENDPOINTS+=("$RPC_ENDPOINT")
  USABLE_HASHES+=("$HASH")
  USABLE_HEADERS+=("$HEADER")
done

if [ "${#USABLE_ENDPOINTS[@]}" -eq 0 ]; then
  echo "No RPC endpoint returned a usable header for ${NETWORK} at height ${REFERENCE_TARGET_HEIGHT_DEC}." >&2
  if [ "${#FAILED_ENDPOINTS[@]}" -gt 0 ]; then
    echo "Ignored RPC endpoints (errors):"
    for i in "${!FAILED_ENDPOINTS[@]}"; do
      echo "  - ${FAILED_ENDPOINTS[$i]}: ${FAILED_REASONS[$i]}"
    done
  fi
  exit 1
fi

echo "Returned hashes at target checkpoint height:"
for i in "${!USABLE_ENDPOINTS[@]}"; do
  echo "  - ${USABLE_ENDPOINTS[$i]} => ${USABLE_HASHES[$i]}"
done

if [ "${#FAILED_ENDPOINTS[@]}" -gt 0 ]; then
  echo "Ignored RPC endpoints (errors):"
  for i in "${!FAILED_ENDPOINTS[@]}"; do
    echo "  - ${FAILED_ENDPOINTS[$i]}: ${FAILED_REASONS[$i]}"
  done
fi

UNIQUE_KEYS=()
UNIQUE_COUNTS=()
UNIQUE_HEADERS=()
UNIQUE_HASHES=()

for i in "${!USABLE_HASHES[@]}"; do
  KEY="${USABLE_HASHES[$i]}${FIELD_SEPARATOR}${USABLE_HEADERS[$i]}"
  HEADER_VALUE="${USABLE_HEADERS[$i]}"
  FOUND="false"
  for u in "${!UNIQUE_KEYS[@]}"; do
    if [ "${UNIQUE_KEYS[$u]}" = "$KEY" ]; then
      (( UNIQUE_COUNTS[$u] += 1 ))
      FOUND="true"
      break
    fi
  done

  if [ "$FOUND" = "false" ]; then
    UNIQUE_KEYS+=("$KEY")
    UNIQUE_COUNTS+=(1)
    UNIQUE_HEADERS+=("$HEADER_VALUE")
    UNIQUE_HASHES+=("${USABLE_HASHES[$i]}")
  fi
done

if [ "${#UNIQUE_KEYS[@]}" -eq 1 ]; then
  REFERENCE_HEADER="${USABLE_HEADERS[0]}"
else
  MAJORITY_COUNT=0
  MAJORITY_INDEX=-1
  for i in "${!UNIQUE_KEYS[@]}"; do
    if [ "${UNIQUE_COUNTS[$i]}" -gt "$MAJORITY_COUNT" ]; then
      MAJORITY_COUNT="${UNIQUE_COUNTS[$i]}"
      MAJORITY_INDEX="$i"
    fi
  done

  USABLE_COUNT="${#USABLE_ENDPOINTS[@]}"
  if [ "$MAJORITY_COUNT" -le $((USABLE_COUNT / 2)) ]; then
    echo "Inconsistent checkpoints detected for ${NETWORK}: no clear majority." >&2
    exit 1
  fi

  echo "Inconsistent checkpoints detected for ${NETWORK}: majority candidate is ${UNIQUE_HASHES[$MAJORITY_INDEX]} (${MAJORITY_COUNT}/${USABLE_COUNT})"
  if [ ! -t 0 ]; then
    echo "Interactive confirmation required. Run this script from a terminal." >&2
    exit 1
  fi

  while true; do
    echo -n "Apply this majority header anyway? [y/N] "
    read -r ANSWER
    case "${ANSWER:-}" in
      [Yy][Ee][Ss]|[Yy])
        REFERENCE_HEADER="${UNIQUE_HEADERS[$MAJORITY_INDEX]}"
        break
        ;;
      [Nn][Oo]|[Nn]|"")
        echo "Aborted by user."
        exit 1
        ;;
      *)
        echo "Please answer y or n."
        ;;
    esac
  done
fi

if [ -z "$REFERENCE_HEADER" ] || [ "$REFERENCE_HEADER" = "null" ]; then
  echo "No valid reference header could be selected." >&2
  exit 1
fi

TMP_CHECKPOINT="$(mktemp "${SPEC_FILE}.tmp.XXXXXX")"
printf '%s\n' "$REFERENCE_HEADER" > "$TMP_CHECKPOINT"
mv "$TMP_CHECKPOINT" "$SPEC_FILE"

echo "Updated checkpoint header for ${NETWORK} in ${SPEC_FILE}"
