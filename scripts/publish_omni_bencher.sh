#!/usr/bin/env bash
set -euo pipefail

if [[ -f "local-sdk.env" ]]; then
  # shellcheck disable=SC1091
  source local-sdk.env
fi

SDK_MIRROR_DIR="${SDK_MIRROR_DIR:-duniter-polkadot-sdk.git}"
SDK_REV="${SDK_COMMIT:-$(grep -m1 'source = "git+https://github.com/duniter/duniter-polkadot-sdk' Cargo.lock | sed -E 's/^.*#([0-9a-f]+)"$/\1/')}"
SDK_REV="${SDK_REV:0:7}"
RUSTC_VERSION="$(rustc --version | awk '{print $2}')"
TARGET_TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"
PKG_NAME="frame-omni-bencher"
PKG_VERSION="${SDK_REV}-${RUSTC_VERSION}-${TARGET_TRIPLE}"
PKG_FILE="${PKG_NAME}-${PKG_VERSION}.gz"
PKG_URL="${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/packages/generic/${PKG_NAME}/${PKG_VERSION}/${PKG_FILE}"

echo "Package key: ${PKG_VERSION}"

PKG_CHECK_HTTP_CODE="$(curl --silent --show-error --location --output /dev/null --write-out '%{http_code}' --header "JOB-TOKEN: ${CI_JOB_TOKEN}" "${PKG_URL}")"
if [[ "${PKG_CHECK_HTTP_CODE}" == "200" ]]; then
  echo "Package already exists, downloading it for downstream artifact reuse."
  mkdir -p target
  curl --silent --show-error --fail --location --header "JOB-TOKEN: ${CI_JOB_TOKEN}" --output target/frame-omni-bencher.gz "${PKG_URL}"
  exit 0
fi
if [[ "${PKG_CHECK_HTTP_CODE}" != "404" && "${PKG_CHECK_HTTP_CODE}" != "403" ]]; then
  echo "Package probe returned HTTP ${PKG_CHECK_HTTP_CODE}; proceeding with publish attempt."
fi

# Keep SDK sources out of Cargo's `target/` tree; wasm-builder metadata
# resolution can break when the workspace itself lives under `target/`.
SDK_SRC_DIR=".ci/duniter-polkadot-sdk-src"
rm -rf "${SDK_SRC_DIR}"
if [[ -n "${SDK_BRANCH:-}" ]]; then
  git clone --quiet --single-branch --branch "${SDK_BRANCH}" "${SDK_MIRROR_DIR}" "${SDK_SRC_DIR}" \
    || git clone --quiet "${SDK_MIRROR_DIR}" "${SDK_SRC_DIR}"
else
  git clone --quiet "${SDK_MIRROR_DIR}" "${SDK_SRC_DIR}"
fi

OMNI_BENCHER_MANIFEST="${SDK_SRC_DIR}/substrate/utils/frame/omni-bencher/Cargo.toml"
if [[ ! -f "${OMNI_BENCHER_MANIFEST}" && -n "${SDK_COMMIT:-}" ]]; then
  git -C "${SDK_SRC_DIR}" fetch --depth 1 origin "${SDK_COMMIT}"
  git -C "${SDK_SRC_DIR}" checkout --detach FETCH_HEAD
fi

if [[ ! -f "${OMNI_BENCHER_MANIFEST}" ]]; then
  echo "frame-omni-bencher Cargo.toml not found at ${OMNI_BENCHER_MANIFEST}"
  exit 1
fi

SKIP_WASM_BUILD=1 ./scripts/cargo_with_vendor.sh build --manifest-path "${OMNI_BENCHER_MANIFEST}" --target-dir target/omni-bencher
gzip -9 -c target/omni-bencher/debug/frame-omni-bencher > "${PKG_FILE}"
cp "${PKG_FILE}" target/frame-omni-bencher.gz

HTTP_CODE="$(curl --silent --show-error --write-out '%{http_code}' --output /tmp/omni-upload-response.txt --header "JOB-TOKEN: ${CI_JOB_TOKEN}" --upload-file "${PKG_FILE}" "${PKG_URL}")"
if [[ "${HTTP_CODE}" == "201" || "${HTTP_CODE}" == "200" || "${HTTP_CODE}" == "409" ]]; then
  echo "Publish result HTTP ${HTTP_CODE}"
elif [[ "${HTTP_CODE}" == "403" ]]; then
  if [[ -n "${CI_MERGE_REQUEST_ID:-}" ]]; then
    echo "No permission to upload package with CI_JOB_TOKEN in MR pipeline (HTTP 403), continuing with local artifact only."
  else
    echo "Failed to upload package on non-MR pipeline (HTTP 403)"
    cat /tmp/omni-upload-response.txt || true
    exit 1
  fi
else
  echo "Failed to upload package (HTTP ${HTTP_CODE})"
  cat /tmp/omni-upload-response.txt || true
  exit 1
fi
