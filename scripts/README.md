# Scripts

Scripts not moved to xtasks because they only depend on Docker and not Rust.

## CI dependency helpers

- `prepare_local_polkadot_sdk.sh [Cargo.lock] [mirror-dir] [Cargo.toml]`: Reads the SDK branch from root `Cargo.toml`, fetches branch tip with `--depth 1`, and checks `Cargo.lock` uses that exact commit.
- `cargo_with_vendor.sh <cargo-args...>`: Runs cargo after rewriting `duniter-polkadot-sdk` git URL to the local mirror prepared in CI.
- `check_toolchain_sync.sh [ci-image]`: Verifies `rust-toolchain.toml`, active CI `rustc`, and srtool release image versions all match the Rust version derived from the CI image tag suffix.
