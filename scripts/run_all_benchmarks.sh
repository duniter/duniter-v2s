#!/usr/bin/env bash
set -euo pipefail

OMNI_BENCHER_BIN="${OMNI_BENCHER_BIN:-$(command -v frame-omni-bencher || true)}"
if [[ -z "$OMNI_BENCHER_BIN" ]]; then
  echo "frame-omni-bencher not found in PATH. Install it first or set OMNI_BENCHER_BIN=/path/to/frame-omni-bencher."
  exit 1
fi

storage_chain_for() {
  case "$1" in
    g1) echo "${STORAGE_CHAIN_G1:-g1_dev}" ;;
    gtest) echo "${STORAGE_CHAIN_GTEST:-gtest_dev}" ;;
    gdev) echo "${STORAGE_CHAIN_GDEV:-gdev}" ;;
    *)
      echo "unsupported chain '$1'" >&2
      exit 1
      ;;
  esac
}

storage_base_path_for() {
  case "$1" in
    g1) echo "${STORAGE_BASE_PATH_G1:-}" ;;
    gtest) echo "${STORAGE_BASE_PATH_GTEST:-}" ;;
    gdev) echo "${STORAGE_BASE_PATH_GDEV:-}" ;;
    *)
      echo "unsupported chain '$1'" >&2
      exit 1
      ;;
  esac
}

STORAGE_STATE_VERSION="${STORAGE_STATE_VERSION:-1}"
STORAGE_MUL="${STORAGE_MUL:-2}"

for chain in g1 gtest gdev; do
  runtime_package="${chain}-runtime"
  runtime_wasm="target/release/wbuild/${runtime_package}/${chain}_runtime.compact.compressed.wasm"
  chain_spec="target/${chain}-dev-spec.json"
  storage_chain="$(storage_chain_for "$chain")"
  storage_base_path="$(storage_base_path_for "$chain")"

  if [[ -z "$storage_base_path" ]]; then
    echo "missing storage snapshot path for ${chain}. Set STORAGE_BASE_PATH_${chain^^} to the chain base path to use with 'duniter benchmark storage'."
    exit 1
  fi

  cargo build --release -p "$runtime_package" --features runtime-benchmarks
  SKIP_WASM_BUILD=1 cargo build --release -p duniter --no-default-features --features "${chain},runtime-benchmarks"
  test -f "$runtime_wasm"
  target/release/duniter build-spec --chain dev --disable-default-bootnode > "$chain_spec"
  test -f "$chain_spec"

  "$OMNI_BENCHER_BIN" \
    v1 benchmark overhead \
    --runtime "$runtime_wasm" \
    --genesis-builder=runtime \
    --genesis-builder-preset=local_testnet \
    --weight-path="./runtime/$chain/src/weights/" \
    --warmup=10 \
    --repeat=100

  "$OMNI_BENCHER_BIN" \
    v1 benchmark pallet \
    --chain "$chain_spec" \
    --genesis-builder=spec-genesis \
    --steps=50 \
    --repeat=20 \
    --pallet="*" \
    --extrinsic="*" \
    --header=./file_header.txt \
    --output="./runtime/$chain/src/weights/"

  target/release/duniter benchmark storage \
    --base-path "$storage_base_path" \
    --chain "$storage_chain" \
    --mul "$STORAGE_MUL" \
    --weight-path "./runtime/$chain/src/weights/" \
    --header ./file_header.txt \
    --state-version "$STORAGE_STATE_VERSION"
done
