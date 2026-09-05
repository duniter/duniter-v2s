#!/usr/bin/env bash
# Source this file so the selected Cargo binary directory stays on PATH.

ensure_subxt() {
    local expected_version="0.50.3"
    local installed_version
    export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"

    installed_version="$(subxt version 2>/dev/null | awk '{print $2}' || true)"
    if [[ "${installed_version%%-*}" != "$expected_version" ]]; then
        cargo install subxt-cli --version "$expected_version" --locked --force || return
        hash -r
        installed_version="$(subxt version | awk '{print $2}')"
    fi

    if [[ "${installed_version%%-*}" != "$expected_version" ]]; then
        echo "Expected subxt-cli $expected_version, found $installed_version" >&2
        return 1
    fi
    echo "Using subxt-cli $installed_version"
}

ensure_subxt
