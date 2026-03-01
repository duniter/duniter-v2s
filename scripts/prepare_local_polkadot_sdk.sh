#!/usr/bin/env bash
set -euo pipefail

LOCK_FILE="${1:-Cargo.lock}"
MIRROR_DIR="${2:-duniter-polkadot-sdk.git}"
MANIFEST_FILE="${3:-Cargo.toml}"
SDK_URL="${DUNITER_POLKADOT_SDK_URL:-https://github.com/duniter/duniter-polkadot-sdk}"

if [[ ! -f "$LOCK_FILE" ]]; then
  echo "Lock file not found: $LOCK_FILE" >&2
  exit 1
fi

if [[ ! -f "$MANIFEST_FILE" ]]; then
  echo "Manifest file not found: $MANIFEST_FILE" >&2
  exit 1
fi

SDK_BRANCH="$(
  grep -m1 "git = 'https://github.com/duniter/duniter-polkadot-sdk'" "$MANIFEST_FILE" \
    | sed -E "s/.*branch = '([^']+)'.*/\1/"
)"

if [[ -z "${SDK_BRANCH:-}" || "$SDK_BRANCH" == *"branch = '"* ]]; then
  echo "Could not find duniter-polkadot-sdk branch in $MANIFEST_FILE" >&2
  exit 1
fi

SDK_SOURCE="$(
  grep -m1 'source = "git+https://github.com/duniter/duniter-polkadot-sdk' "$LOCK_FILE" \
    | sed -E 's/^.*"git\+([^"]+)".*$/\1/'
)"

if [[ -z "${SDK_SOURCE:-}" ]]; then
  echo "Could not find duniter-polkadot-sdk git source in $LOCK_FILE" >&2
  exit 1
fi

SDK_COMMIT="${SDK_SOURCE##*#}"
QUERY_PART="${SDK_SOURCE#*\?}"
if [[ "$QUERY_PART" == "$SDK_SOURCE" ]]; then
  QUERY_PART=""
fi
QUERY_PART="${QUERY_PART%%#*}"

LOCK_BRANCH=""
if [[ -n "$QUERY_PART" ]]; then
  IFS='&' read -r -a QUERY_ITEMS <<< "$QUERY_PART"
  for item in "${QUERY_ITEMS[@]}"; do
    if [[ "$item" == branch=* ]]; then
      LOCK_BRANCH="${item#branch=}"
      break
    fi
  done
fi

if [[ -n "$LOCK_BRANCH" && "$LOCK_BRANCH" != "$SDK_BRANCH" ]]; then
  echo "Branch mismatch: Cargo.toml uses '$SDK_BRANCH' but Cargo.lock source uses '$LOCK_BRANCH'." >&2
  echo "Run cargo update after changing SDK branch." >&2
  exit 1
fi

rm -rf "$MIRROR_DIR"
git init --bare "$MIRROR_DIR"
git -C "$MIRROR_DIR" remote add origin "$SDK_URL"

# Always fetch only the current branch tip (depth=1).
# CI assumes Cargo.lock is kept in sync with the latest commit of this maintained branch.
git -C "$MIRROR_DIR" fetch --depth 1 origin "$SDK_BRANCH"
BRANCH_HEAD_COMMIT="$(git -C "$MIRROR_DIR" rev-parse FETCH_HEAD)"

if [[ "$SDK_COMMIT" != "$BRANCH_HEAD_COMMIT" ]]; then
  echo "Cargo.lock uses SDK commit $SDK_COMMIT, but branch '$SDK_BRANCH' tip is $BRANCH_HEAD_COMMIT." >&2
  echo "Update Cargo.lock to the latest branch commit before running CI." >&2
  exit 1
fi

printf "SDK_MIRROR_DIR=%s\n" "$MIRROR_DIR" > local-sdk.env
printf "SDK_URL=%s\n" "$SDK_URL" >> local-sdk.env
printf "SDK_BRANCH=%s\n" "$SDK_BRANCH" >> local-sdk.env
printf "SDK_COMMIT=%s\n" "$BRANCH_HEAD_COMMIT" >> local-sdk.env

echo "Prepared local SDK mirror: dir=$MIRROR_DIR branch=$SDK_BRANCH commit=$BRANCH_HEAD_COMMIT"
