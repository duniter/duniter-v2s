#!/usr/bin/env bash
set -euo pipefail

SDK_MIRROR_DIR="${SDK_MIRROR_DIR:-duniter-polkadot-sdk.git}"

if [[ -f "local-sdk.env" ]]; then
  # shellcheck disable=SC1091
  source local-sdk.env
fi

if [[ ! -d "$SDK_MIRROR_DIR" ]]; then
  echo "Missing local SDK mirror '$SDK_MIRROR_DIR'. Run the sdk_resolve CI job first." >&2
  exit 1
fi

SDK_MIRROR_ABS="$(cd "$SDK_MIRROR_DIR" && pwd)"
SDK_FILE_URL="file://${SDK_MIRROR_ABS}"

git config --global url."${SDK_FILE_URL}".insteadOf "https://github.com/duniter/duniter-polkadot-sdk"
git config --global url."${SDK_FILE_URL}".insteadOf "git@github.com:duniter/duniter-polkadot-sdk"
git config --global url."${SDK_FILE_URL}".insteadOf "ssh://git@github.com/duniter/duniter-polkadot-sdk"

export CARGO_NET_GIT_FETCH_WITH_CLI=true

exec cargo "$@"
