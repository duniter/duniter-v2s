#!/usr/bin/env bash
set -euo pipefail

echo "Building g1 runtime wasm with runtime-benchmarks..."
./scripts/cargo_with_vendor.sh build --release -p g1-runtime --features runtime-benchmarks

echo "Building duniter benchmark binary..."
SKIP_WASM_BUILD=1 ./scripts/cargo_with_vendor.sh build -p duniter --no-default-features --features g1,runtime-benchmarks

if [[ -f "local-sdk.env" ]]; then
  # shellcheck disable=SC1091
  source local-sdk.env
else
  echo "local-sdk.env not found; falling back to Cargo.lock for SDK revision."
fi

SDK_REV="${SDK_COMMIT:-$(grep -m1 'source = "git+https://github.com/duniter/duniter-polkadot-sdk' Cargo.lock | sed -E 's/^.*#([0-9a-f]+)"$/\1/')}"
if [[ -z "${SDK_REV:-}" ]]; then
  echo "Could not resolve SDK revision from local-sdk.env or Cargo.lock." >&2
  exit 1
fi
SDK_REV="${SDK_REV:0:7}"
RUSTC_VERSION="$(rustc --version | awk '{print $2}')"
TARGET_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
PKG_NAME="frame-omni-bencher"
PKG_VERSION="${SDK_REV}-${RUSTC_VERSION}-${TARGET_TRIPLE}"
PKG_FILE="${PKG_NAME}-${PKG_VERSION}.gz"
PKG_URL="${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/packages/generic/${PKG_NAME}/${PKG_VERSION}/${PKG_FILE}"

echo "Resolved omni-bencher package key: ${PKG_VERSION}"

mkdir -p target
if [[ -f "target/frame-omni-bencher.gz" ]]; then
  echo "Using frame-omni-bencher artifact from upstream job."
else
  echo "Artifact missing; downloading ${PKG_URL}"
  curl --silent --show-error --fail --location --header "JOB-TOKEN: ${CI_JOB_TOKEN}" --output target/frame-omni-bencher.gz "${PKG_URL}"
fi

echo "Expanding frame-omni-bencher binary..."
gzip -d -c target/frame-omni-bencher.gz > target/frame-omni-bencher
chmod +x target/frame-omni-bencher

CHAIN_SPEC="target/g1-dev-spec.json"
RUNTIME_WASM="target/release/wbuild/g1-runtime/g1_runtime.compact.compressed.wasm"

if [[ ! -f "${RUNTIME_WASM}" ]]; then
  echo "Expected runtime wasm not found: ${RUNTIME_WASM}" >&2
  echo "Available wbuild outputs:" >&2
  find target -path '*/wbuild/*' | sed -n '1,80p' >&2 || true
  exit 1
fi

echo "Using runtime wasm: ${RUNTIME_WASM}"

echo "Generating dev chain spec..."
WASM_FILE="${RUNTIME_WASM}" target/debug/duniter build-spec --chain dev --disable-default-bootnode > "${CHAIN_SPEC}"
if [[ ! -f "${CHAIN_SPEC}" ]]; then
  echo "Expected chain spec not found: ${CHAIN_SPEC}" >&2
  exit 1
fi

echo "Running overhead benchmark smoke test..."
target/frame-omni-bencher \
  v1 benchmark overhead \
  --runtime "${RUNTIME_WASM}" \
  --genesis-builder=runtime \
  --genesis-builder-preset=local_testnet \
  --warmup=1 \
  --repeat=1 \
  --weight-path=./runtime/g1/src/weights/

echo "Running pallet benchmark smoke test..."
target/frame-omni-bencher \
  v1 benchmark pallet \
  --chain "${CHAIN_SPEC}" \
  --genesis-builder=spec-genesis \
  --steps=2 \
  --repeat=1 \
  --pallet="*" \
  --extrinsic="*" \
  --output=./runtime/g1/src/weights/

echo "Rebuilding g1 runtime to validate generated weights..."
./scripts/cargo_with_vendor.sh build -p g1-runtime
