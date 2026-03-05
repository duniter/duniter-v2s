#!/usr/bin/env bash
set -euo pipefail

NODE_PID=""

cleanup() {
  if [ -n "${NODE_PID:-}" ] && kill -0 "$NODE_PID" 2>/dev/null; then
    kill "$NODE_PID" 2>/dev/null || true
    wait "$NODE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

BUILD_PROFILE="${DUNITER_BUILD_PROFILE:-debug}"
case "$BUILD_PROFILE" in
  debug)
    BUILD_FLAG=""
    TARGET_PROFILE_DIR="debug"
    ;;
  release)
    BUILD_FLAG="--release"
    TARGET_PROFILE_DIR="release"
    ;;
  *)
    echo "Error: invalid DUNITER_BUILD_PROFILE='$BUILD_PROFILE' (expected 'debug' or 'release')."
    exit 1
    ;;
esac

if ! command -v subxt >/dev/null 2>&1; then
  cargo install subxt-cli
fi

needs_regen=0
for chain in g1 gtest gdev; do
  cargo build $BUILD_FLAG --no-default-features --features "$chain"
  "./target/${TARGET_PROFILE_DIR}/duniter" --dev --tmp &
  NODE_PID=$!
  sleep 20
  subxt metadata -f bytes > "resources/${chain}_new_metadata.scale"
  kill "$NODE_PID" 2>/dev/null || true
  wait "$NODE_PID" 2>/dev/null || true
  NODE_PID=""

  if ! cmp -s "resources/${chain}_new_metadata.scale" "resources/${chain}_metadata.scale"; then
    needs_regen=1
  fi
done

if [ "$needs_regen" -eq 1 ]; then
  echo "Metadata file needs to be generated. How to do it? scripts/generate_all_metadata.sh"
  exit 1
fi

echo "Metadata files are up to date."
