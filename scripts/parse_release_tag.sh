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
  if ! command -v jq >/dev/null 2>&1; then
    echo "jq is required to resolve RAW_SPEC_URL in release jobs"
    exit 1
  fi

  PROJECT_ID="${CI_PROJECT_ID:-nodes%2Frust%2Fduniter-v2s}"
  RELEASES_API="${CI_API_V4_URL:-https://git.duniter.org/api/v4}/projects/${PROJECT_ID}/releases?per_page=100"
  CURL_ARGS=(-fsSL)

  if [ -n "${GITLAB_TOKEN:-}" ]; then
    CURL_ARGS+=(--header "PRIVATE-TOKEN: ${GITLAB_TOKEN}")
  elif [ -n "${CI_JOB_TOKEN:-}" ]; then
    CURL_ARGS+=(--header "JOB-TOKEN: ${CI_JOB_TOKEN}")
  fi

  RELEASES_JSON="$(curl "${CURL_ARGS[@]}" "${RELEASES_API}")"
  RAW_SPEC_URL="$(
    printf '%s' "${RELEASES_JSON}" | jq -r \
      --arg prefix "${NETWORK}-" \
      --arg asset "${RUNTIME}-raw.json" \
      '
        sort_by(.released_at // .created_at) | reverse
        | map(select(.tag_name | startswith($prefix)))
        | map(.assets.links[]? | select(.name == $asset) | .url)
        | .[0] // empty
      '
  )"

  if [ -z "${RAW_SPEC_URL}" ]; then
    echo "Could not find ${RUNTIME}-raw.json in any release matching prefix ${NETWORK}-"
    exit 1
  fi
fi

export RELEASE_TAG
export NETWORK
export RUNTIME
export CLIENT_VERSION
export RAW_SPEC_URL
