#!/usr/bin/env bash

set -euo pipefail

if [ -z "${RUNTIME:-}" ]; then
  echo "RUNTIME is required"
  exit 1
fi

if [ -z "${RAW_SPEC_URL:-}" ]; then
  echo "RAW_SPEC_URL is required"
  exit 1
fi

client_spec_path="node/specs/${RUNTIME}_client-specs.yaml"
raw_spec_path="node/specs/${RUNTIME}-raw.json"

if [ ! -f "${client_spec_path}" ]; then
  echo "Client spec not found: ${client_spec_path}"
  exit 1
fi

mkdir -p node/specs

downloaded_raw_spec="$(mktemp)"
patched_raw_spec="$(mktemp)"
trap 'rm -f "${downloaded_raw_spec}" "${patched_raw_spec}"' EXIT

echo "Downloading ${RUNTIME}-raw.json from ${RAW_SPEC_URL}"
curl -fSL --retry 3 --retry-delay 5 -o "${downloaded_raw_spec}" "${RAW_SPEC_URL}"

if [ ! -s "${downloaded_raw_spec}" ]; then
  echo "Downloaded raw spec is empty: ${RAW_SPEC_URL}"
  exit 1
fi

boot_nodes_json="$(
  awk '
    /^bootNodes:[[:space:]]*$/ { in_boot_nodes=1; next }
    in_boot_nodes && /^[^[:space:]]/ { in_boot_nodes=0 }
    in_boot_nodes && /^[[:space:]]*-[[:space:]]*/ {
      sub(/^[[:space:]]*-[[:space:]]*/, "", $0)
      if ($0 ~ /^".*"$/) {
        sub(/^"/, "", $0)
        sub(/"$/, "", $0)
      }
      print
    }
  ' "${client_spec_path}" | jq -Rsc 'split("\n") | map(select(length > 0))'
)"

jq --argjson bootNodes "${boot_nodes_json}" \
  '.bootNodes = $bootNodes' \
  "${downloaded_raw_spec}" > "${patched_raw_spec}"

mv "${patched_raw_spec}" "${raw_spec_path}"

echo "Patched top-level bootNodes in ${raw_spec_path} from ${client_spec_path}"
