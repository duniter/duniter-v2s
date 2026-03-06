#!/usr/bin/env bash

set -euo pipefail

if [ -z "${CI_COMMIT_TAG:-}" ]; then
  echo "CI_COMMIT_TAG is required for release jobs"
  exit 1
fi

if ! printf '%s\n' "$CI_COMMIT_TAG" | grep -Eq '^[a-zA-Z0-9]+-[0-9]+-[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "Invalid release tag '${CI_COMMIT_TAG}' (expected SPEC_NAME-SPEC_VERSION-BINARY_VERSION, e.g. g1-1200-2.0.0)"
  exit 1
fi

RELEASE_TAG="$CI_COMMIT_TAG"
NETWORK="${RELEASE_TAG%%-*}-$(echo "$RELEASE_TAG" | cut -d- -f2)"
CLIENT_VERSION_FROM_TAG=$(echo "$RELEASE_TAG" | cut -d- -f3)
CLIENT_VERSION="$(grep '^version = ' node/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')"

if [ "$CLIENT_VERSION_FROM_TAG" != "$CLIENT_VERSION" ]; then
  echo "Tag binary version '${CLIENT_VERSION_FROM_TAG}' does not match Cargo.toml version '${CLIENT_VERSION}'"
  exit 1
fi

export RELEASE_TAG
export NETWORK
export CLIENT_VERSION
