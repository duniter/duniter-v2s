#!/usr/bin/env bash

set -euo pipefail

DRY_RUN="${DRY_RUN:-0}"

is_dry_run() {
  [ "${DRY_RUN}" = "1" ] || [ "${DRY_RUN}" = "true" ] || [ "${DRY_RUN}" = "yes" ]
}

required_vars=(
  CI_API_V4_URL
  CI_COMMIT_TAG
  CI_PROJECT_ID
)

if ! is_dry_run; then
  required_vars+=(
    CI_COMMIT_SHA
    CI_PROJECT_URL
  )
fi

for var_name in "${required_vars[@]}"; do
  if [ -z "${!var_name:-}" ]; then
    echo "${var_name} is required"
    exit 1
  fi
done

if [ -n "${CI_JOB_TOKEN:-}" ]; then
  GITLAB_AUTH_HEADER_NAME="JOB-TOKEN"
  GITLAB_AUTH_HEADER_VALUE="${CI_JOB_TOKEN}"
elif [ -n "${GITLAB_TOKEN:-}" ]; then
  GITLAB_AUTH_HEADER_NAME="PRIVATE-TOKEN"
  GITLAB_AUTH_HEADER_VALUE="${GITLAB_TOKEN}"
else
  echo "CI_JOB_TOKEN or GITLAB_TOKEN is required"
  exit 1
fi

if ! printf '%s\n' "${CI_COMMIT_TAG}" | grep -Eq '^[a-zA-Z0-9]+-[0-9]+-[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "Invalid release tag '${CI_COMMIT_TAG}'"
  echo "Expected format: NETWORK-GENESIS_RUNTIME_VERSION-BINARY_VERSION (e.g. g1-1100-2.0.0)"
  exit 1
fi

NETWORK="${CI_COMMIT_TAG%%-*}-$(printf '%s' "${CI_COMMIT_TAG}" | cut -d- -f2)"
CLIENT_VERSION="$(printf '%s' "${CI_COMMIT_TAG}" | cut -d- -f3)"

gitlab_get() {
  local url="$1"
  curl --fail --silent --show-error \
    --header "${GITLAB_AUTH_HEADER_NAME}: ${GITLAB_AUTH_HEADER_VALUE}" \
    "${url}"
}

gitlab_try_get() {
  local url="$1"
  curl --fail --silent \
    --header "${GITLAB_AUTH_HEADER_NAME}: ${GITLAB_AUTH_HEADER_VALUE}" \
    "${url}"
}

build_changes_section() {
  local milestone
  local encoded_milestone
  local encoded_label
  local mr_json
  local -a change_lines=()

  milestone="client-${CLIENT_VERSION}"
  encoded_milestone="$(jq -rn --arg v "${milestone}" '$v|@uri')"
  encoded_label="$(jq -rn --arg v "RN-binary" '$v|@uri')"
  mr_json="$(
    gitlab_try_get "${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/merge_requests?state=merged&scope=all&milestone=${encoded_milestone}&labels=${encoded_label}&per_page=100&order_by=updated_at&sort=asc"
  )"

  if [ -z "${mr_json}" ]; then
    printf '## Changes\n\n'
    printf '%s\n' ' - Failed to retrieve merged RN-binary MRs automatically; please edit this section manually.'
    return
  fi

  while IFS= read -r line; do
    [ -n "${line}" ] || continue
    change_lines+=("${line}")
  done < <(
    printf '%s' "${mr_json}" \
      | jq -r '.[] | " - \(.title) (!\(.iid))"'
  )

  printf '## Changes\n\n'
  if [ "${#change_lines[@]}" -eq 0 ]; then
    printf '%s\n' ' - None'
  else
    printf '%s\n' "${change_lines[@]}"
  fi
}

release_assets_dir="release/client-assets"

changes_section="$(build_changes_section)"

release_description=$(
  cat <<EOF
Client release artifacts for \`${NETWORK}\`.

Packages:
- Debian amd64
- Debian arm64
- RPM x86_64
- RPM aarch64

${changes_section}
EOF
)

if is_dry_run; then
  printf '%s\n' "${release_description}"
  exit 0
fi

mkdir -p "${release_assets_dir}"

shopt -s nullglob

deb_amd64=(target/debian/*_"${CLIENT_VERSION}"-*_amd64.deb)
deb_arm64=(target/debian/*_"${CLIENT_VERSION}"-*_arm64.deb)
rpm_x86_64=(target/generate-rpm/*-"${CLIENT_VERSION}"-*.x86_64.rpm)
rpm_aarch64=(target/generate-rpm/*-"${CLIENT_VERSION}"-*.aarch64.rpm)

assert_single_match() {
  local label="$1"
  shift
  local matches=("$@")

  if [ "${#matches[@]}" -ne 1 ]; then
    echo "Expected exactly one ${label} artifact, found ${#matches[@]}"
    printf 'Matches:\n'
    printf '  %s\n' "${matches[@]:-<none>}"
    exit 1
  fi
}

assert_single_match "amd64 deb" "${deb_amd64[@]}"
assert_single_match "arm64 deb" "${deb_arm64[@]}"
assert_single_match "x86_64 rpm" "${rpm_x86_64[@]}"
assert_single_match "aarch64 rpm" "${rpm_aarch64[@]}"

package_files=(
  "${deb_amd64[0]}"
  "${deb_arm64[0]}"
  "${rpm_x86_64[0]}"
  "${rpm_aarch64[0]}"
)

for package_file in "${package_files[@]}"; do
  cp "${package_file}" "${release_assets_dir}/"
done

release_tag_encoded="$(jq -rn --arg v "${CI_COMMIT_TAG}" '$v|@uri')"
release_api="${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/releases"
release_url="${release_api}/${release_tag_encoded}"

release_status="$(
  curl --silent --output /dev/null --write-out '%{http_code}' \
    --header "${GITLAB_AUTH_HEADER_NAME}: ${GITLAB_AUTH_HEADER_VALUE}" \
    "${release_url}"
)"

if [ "${release_status}" = "404" ]; then
  echo "Creating GitLab release ${CI_COMMIT_TAG}"
  curl --fail --silent --show-error \
    --request POST \
    --header "${GITLAB_AUTH_HEADER_NAME}: ${GITLAB_AUTH_HEADER_VALUE}" \
    --data-urlencode "name=${CI_COMMIT_TAG}" \
    --data-urlencode "tag_name=${CI_COMMIT_TAG}" \
    --data-urlencode "ref=${CI_COMMIT_SHA}" \
    --data-urlencode "description=${release_description}" \
    "${release_api}" >/dev/null
elif [ "${release_status}" = "200" ]; then
  echo "Updating GitLab release ${CI_COMMIT_TAG}"
  curl --fail --silent --show-error \
    --request PUT \
    --header "${GITLAB_AUTH_HEADER_NAME}: ${GITLAB_AUTH_HEADER_VALUE}" \
    --data-urlencode "name=${CI_COMMIT_TAG}" \
    --data-urlencode "description=${release_description}" \
    "${release_url}" >/dev/null
else
  echo "Unexpected status while querying release ${CI_COMMIT_TAG}: ${release_status}"
  exit 1
fi

existing_links_json="$(
  gitlab_get "${release_url}"
)"

for package_file in "${release_assets_dir}"/*; do
  asset_name="$(basename "${package_file}")"
  if printf '%s' "${existing_links_json}" | jq -e --arg name "${asset_name}" '.assets.links[]? | select(.name == $name)' >/dev/null; then
    echo "Asset link already exists: ${asset_name}"
    continue
  fi

  asset_url="${CI_PROJECT_URL}/-/jobs/artifacts/${CI_COMMIT_TAG}/raw/${release_assets_dir}/${asset_name}?job=release_gitlab_page"

  echo "Creating release asset link for ${asset_name}"
  curl --fail --silent --show-error \
    --request POST \
    --header "${GITLAB_AUTH_HEADER_NAME}: ${GITLAB_AUTH_HEADER_VALUE}" \
    --data-urlencode "name=${asset_name}" \
    --data-urlencode "url=${asset_url}" \
    --data-urlencode "link_type=package" \
    "${release_url}/assets/links" >/dev/null
done
