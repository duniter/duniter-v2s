#!/usr/bin/env bash
set -euo pipefail

CI_IMAGE="${1:-paritytech/ci-unified:bullseye-1.88.0}"

if [[ "$CI_IMAGE" != *:* ]]; then
  echo "Invalid CI image format: $CI_IMAGE" >&2
  exit 1
fi

CI_TAG="${CI_IMAGE##*:}"
EXPECTED_RUST_VERSION="${CI_TAG##*-}"

if [[ ! "$EXPECTED_RUST_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Could not derive Rust version from CI image tag: $CI_TAG" >&2
  exit 1
fi

TOOLCHAIN_VERSION="$(awk -F'"' '/^channel = "/ {print $2; exit}' rust-toolchain.toml)"
if [[ -z "${TOOLCHAIN_VERSION:-}" ]]; then
  echo "Could not read channel from rust-toolchain.toml" >&2
  exit 1
fi

if [[ "$TOOLCHAIN_VERSION" != "$EXPECTED_RUST_VERSION" ]]; then
  echo "rust-toolchain mismatch: channel=$TOOLCHAIN_VERSION expected=$EXPECTED_RUST_VERSION (from $CI_IMAGE)" >&2
  exit 1
fi

RUSTC_VERSION="$(rustc -V | awk '{print $2}')"
if [[ "$RUSTC_VERSION" != "$EXPECTED_RUST_VERSION" ]]; then
  echo "CI rustc mismatch: rustc=$RUSTC_VERSION expected=$EXPECTED_RUST_VERSION (from $CI_IMAGE)" >&2
  exit 1
fi

if ! grep -Eq '^targets = \[ "wasm32v1-none" \]$' rust-toolchain.toml; then
  echo "rust-toolchain target mismatch: expected targets = [ \"wasm32v1-none\" ]" >&2
  exit 1
fi

mapfile -t SRTOOL_VERSIONS < <(
  grep -Eho 'paritytech/srtool:[0-9]+\.[0-9]+\.[0-9]+' \
    xtask/src/runtime/build_runtime.rs \
    xtask/src/network/build_network_runtime.rs \
    | sed 's/.*://' \
    | sort -u
)

if [[ "${#SRTOOL_VERSIONS[@]}" -eq 0 ]]; then
  echo "No srtool image version found in release xtask files." >&2
  exit 1
fi

if [[ "${#SRTOOL_VERSIONS[@]}" -ne 1 ]]; then
  echo "Multiple srtool versions found: ${SRTOOL_VERSIONS[*]}" >&2
  exit 1
fi

if [[ "${SRTOOL_VERSIONS[0]}" != "$EXPECTED_RUST_VERSION" ]]; then
  echo "srtool mismatch: srtool=${SRTOOL_VERSIONS[0]} expected=$EXPECTED_RUST_VERSION (from $CI_IMAGE)" >&2
  exit 1
fi

echo "Toolchain sync OK: rust-toolchain=$TOOLCHAIN_VERSION rustc=$RUSTC_VERSION srtool=${SRTOOL_VERSIONS[0]} ci_image=$CI_IMAGE"
