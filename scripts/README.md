# Scripts

Scripts not moved to xtasks because they only depend on Docker and not Rust.

## Local development helpers

- `run-local-chain.sh [--runtime gdev|gtest|g1] [--technical-committee-members N] [-- <duniter-args...>]`: Builds and runs a local chain with the selected runtime feature using `<runtime>_local` chainspec. It always enforces `--validator --unsafe-force-node-key-generation --sealing manual --tmp`. The optional technical committee size maps to `DUNITER_LOCAL_TECHNICAL_COMMITTEE_MEMBERS`.

## CI dependency helpers

- `prepare_local_polkadot_sdk.sh [Cargo.lock] [mirror-dir] [Cargo.toml]`: Reads the SDK branch from root `Cargo.toml`, fetches branch tip with `--depth 1`, and checks `Cargo.lock` uses that exact commit.
- `cargo_with_vendor.sh <cargo-args...>`: Runs cargo after rewriting `duniter-polkadot-sdk` git URL to the local mirror prepared in CI.
- `check_toolchain_sync.sh [ci-image]`: Verifies `rust-toolchain.toml`, active CI `rustc`, and srtool release image versions all match the Rust version derived from the CI image tag suffix.
- `update-warp-checkpoint.sh <network>`: Downloads `https://git.duniter.org/nodes/networks/-/raw/master/{network}.json` for `g1`, `gdev` or `gtest`, picks a reference finalized head, targets block height `finalized - 10`, reports each `rpc` endpoint’s hash at that height, compares usable headers, and writes `node/specs/{network}-checkpoint.json` (or asks confirmation to apply the majority when inconsistent).
