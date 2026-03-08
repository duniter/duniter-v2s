#!/usr/bin/env bash

set -euo pipefail

if [ -z "${CI_COMMIT_TAG:-}" ]; then
  echo "CI_COMMIT_TAG is required for release jobs"
  exit 1
fi

if ! printf '%s\n' "$CI_COMMIT_TAG" | grep -Eq '^[a-zA-Z0-9]+-[0-9]+-[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "Invalid release tag '${CI_COMMIT_TAG}' (expected NETWORK-GENESIS_RUNTIME_VERSION-BINARY_VERSION, e.g. g1-1100-2.0.0)"
  exit 1
fi

RELEASE_TAG="$CI_COMMIT_TAG"
NETWORK="${RELEASE_TAG%%-*}-$(echo "$RELEASE_TAG" | cut -d- -f2)"
RUNTIME="${NETWORK%%-*}"
CLIENT_VERSION_FROM_TAG=$(echo "$RELEASE_TAG" | cut -d- -f3)
CLIENT_VERSION="$(grep '^version = ' node/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"

if [ "$CLIENT_VERSION_FROM_TAG" != "$CLIENT_VERSION" ]; then
  echo "Tag binary version '${CLIENT_VERSION_FROM_TAG}' does not match Cargo.toml version '${CLIENT_VERSION}'"
  exit 1
fi

if [ -z "${RAW_SPEC_URL:-}" ]; then
  GENESIS_RELEASE_TAG="${NETWORK}"
  PROJECT_ID="${CI_PROJECT_ID:-520}"
  RELEASE_LINKS_API="${CI_API_V4_URL:-https://git.duniter.org/api/v4}/projects/${PROJECT_ID}/releases/${GENESIS_RELEASE_TAG}/assets/links"
  RELEASE_LINKS_JSON="$(curl -fsSL "${RELEASE_LINKS_API}")"
  RAW_SPEC_URL="$(
    printf '%s' "${RELEASE_LINKS_JSON}" \
      | jq -r \
        --arg asset "${RUNTIME}-raw.json" \
        'map(select(.name == $asset) | (.direct_asset_url // .url)) | .[0] // empty'
  )"

  if [ -z "${RAW_SPEC_URL}" ]; then
    echo "Could not find public link to ${RUNTIME}-raw.json via ${RELEASE_LINKS_API}"
    exit 1
  fi
fi

export RELEASE_TAG
export NETWORK
export RUNTIME
export CLIENT_VERSION
export RAW_SPEC_URL
