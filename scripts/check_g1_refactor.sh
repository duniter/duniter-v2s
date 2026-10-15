#!/usr/bin/env bash
# Build and compare the pre-refactor and current production G1 runtimes.
set -euo pipefail
cd "$(dirname "$0")/.."

# Refuse settings that can silently reuse a stale or substituted Wasm artifact.
if [[ -n "${SKIP_WASM_BUILD+x}" || -n "${SKIP_G1_RUNTIME_WASM_BUILD+x}" || -n "${RUNTIME_METADATA_HASH+x}" ]]; then
    echo 'Unset SKIP_WASM_BUILD, SKIP_G1_RUNTIME_WASM_BUILD and RUNTIME_METADATA_HASH.' >&2
    exit 1
fi
base_ref="${1:-${CI_MERGE_REQUEST_DIFF_BASE_SHA:-$(git merge-base HEAD origin/master)}}"
base_sha="$(git rev-parse "${base_ref}^{commit}")"
report="$(pwd)/target/g1-refactor-comparison"
mkdir -p "$report"
rm -f "$report/differential.txt" "$report/before.scale" "$report/after.scale"
source_dir="$(mktemp -d "${TMPDIR:-/tmp}/g1-before.XXXXXX")"
trap 'rm -rf "$source_dir"' EXIT
# A source archive avoids modifying either checkout or its Git refs.
git archive "$base_sha" | tar -x -C "$source_dir"
printf '%s\n' "$base_sha" > "$report/base-sha.txt"
git rev-parse HEAD > "$report/head-sha.txt"
git diff --binary HEAD > "$report/working-tree.patch"

# The SDK artifact must be prepared with sdk_resolve before this command.
if [[ ! -f local-sdk.env ]]; then
    echo 'Run scripts/prepare_local_polkadot_sdk.sh Cargo.lock duniter-polkadot-sdk.git first.' >&2
    exit 1
fi
# Use separate target directories: Cargo can otherwise reuse path-dependency
# artifacts from the other checkout, including macro definitions.
export CARGO_TARGET_DIR="$(pwd)/target/g1-refactor-before"
./scripts/cargo_with_vendor.sh build --manifest-path "$source_dir/Cargo.toml" --locked -p g1-runtime --features metadata-hash
cp "$CARGO_TARGET_DIR/debug/wbuild/g1-runtime/g1_runtime.wasm" "$report/before.wasm"
export CARGO_TARGET_DIR="$(pwd)/target"
./scripts/cargo_with_vendor.sh build --locked -p g1-runtime --features metadata-hash
cp target/debug/wbuild/g1-runtime/g1_runtime.wasm "$report/after.wasm"

G1_BEFORE_WASM="$report/before.wasm" \
G1_AFTER_WASM="$report/after.wasm" \
G1_COMPARISON_DIR="$report" \
    ./scripts/cargo_with_vendor.sh test --locked -p duniter --no-default-features --features g1 \
    --test g1_runtime_differential -- --ignored --nocapture
source scripts/ensure_subxt.sh
python3 scripts/check_runtime_compatibility.py --before-g1 "$report/before.scale" --g1 "$report/after.scale"
echo "G1 comparison passed. Evidence: $report"
