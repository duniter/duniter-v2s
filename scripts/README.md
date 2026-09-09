# Scripts

Scripts not moved to xtasks because they only depend on Docker and not Rust.

## Local development helpers

- `run-local-chain.sh [--runtime gdev|gtest|g1] [--technical-committee-members N] [-- <duniter-args...>]`: Builds and runs a local chain with the selected runtime feature using `<runtime>_local` chainspec. It always enforces `--validator --unsafe-force-node-key-generation --sealing manual --tmp`. The optional technical committee size maps to `DUNITER_LOCAL_TECHNICAL_COMMITTEE_MEMBERS`.

## CI dependency helpers

- `source scripts/ensure_subxt.sh`: Selects subxt-cli 0.50.3, installs it with locked dependencies if needed, and puts the Cargo binary directory on `PATH`. Metadata scripts use this helper locally and in CI.
- `prepare_local_polkadot_sdk.sh [Cargo.lock] [mirror-dir] [Cargo.toml]`: Reads the SDK branch from root `Cargo.toml`, fetches branch tip with `--depth 1`, and checks `Cargo.lock` uses that exact commit.
- `cargo_with_vendor.sh <cargo-args...>`: Runs cargo after rewriting `duniter-polkadot-sdk` git URL to the local mirror prepared in CI.
- `check_toolchain_sync.sh [ci-image]`: Verifies `rust-toolchain.toml`, active CI `rustc`, and srtool release image versions all match the Rust version derived from the CI image tag suffix.
- `update-warp-checkpoint.sh <network>`: Downloads `https://git.duniter.org/nodes/networks/-/raw/master/{network}.json` for `g1`, `gdev` or `gtest`, picks a reference finalized head, targets block height `finalized - 10`, reports each `rpc` endpoint’s hash at that height, compares usable headers, and writes `node/specs/{network}-checkpoint.json` (or asks confirmation to apply the majority when inconsistent).

## Audit production dependencies

Install Python 3.11 or later and the audit tools, then prepare the SDK and run:

```sh
cargo install --locked cargo-auditable --version 0.7.5
cargo install --locked cargo-audit --version 0.22.2
cargo install --locked rust-audit-info --version 0.5.4
./scripts/prepare_local_polkadot_sdk.sh
RUST_LOG=info METADATA_CHAINS=g1 DUNITER_BUILD_PROFILE=release ./scripts/generate_all_metadata.sh
./scripts/audit-production.py
```

The script builds `duniter` and `distance-oracle` in release mode with the pinned
toolchain and lockfile. It uses the Docker release features for G1:
`g1,embed` for the node and `g1,standalone,std` for the oracle, with default
features disabled. Use `--chain gdev` or `--chain gtest` for another network.
Generate metadata for that chain before running the audit. The metadata helper
uses subxt and a temporary local node, as in CI.
Provide its raw chain spec in `node/specs/` before building the embedded node.
The default target is the compiler's host. `--target <triple>` requires the
corresponding Rust target, native libraries, and linker.

Each run saves reports under `target/audit/production/run-*`. The script checks
embedded metadata with `rust-audit-info` and audits both executables. It then
audits a temporary lockfile containing only packages reported by Cargo as build
artifacts, including cached artifacts, build scripts, and procedural macros.
`compiled-packages.json` maps each package to its binaries.
`audit-compiled.txt` and `audit-compiled.json` contain the confirmed-package audit.
Each report directory also preserves the repository's `audit.toml` policy.
The repository's `Cargo.lock` remains unchanged.

`audit-native-rustc.py` bypasses cargo-auditable only for Wasm compiler calls.
The SDK still builds the runtime normally. Native executables retain embedded
audit metadata and the root build retains `--locked`. This avoids applying
cargo-auditable's locked metadata query to the SDK's generated Wasm workspace.
An existing `RUSTC_WRAPPER` environment setting is forwarded.

Embedded metadata can overestimate dependencies on stable Cargo. Entries without
a matching build artifact are listed in each binary's `metadata-surplus.json`
report and excluded from the confirmed-package audit. Exit status is 1 for
confirmed vulnerabilities, unsoundness advisories, or yanked crates, and 2 for
build or audit errors. Maintenance notices remain warnings. The policy is in
`.cargo/audit.toml`, including justified advisory exceptions. The
[review attached to the dependency-audit MR](https://git.duniter.org/-/project/520/uploads/9f036033d5ec0fef59e343b89a584ba5/dependency-vulnerabilities.md)
documents callers, feature prerequisites, and patch assessments.

Cargo may also reuse embedded metadata after lockfile changes that affect only
inactive dependencies. If the direct binary scan lists obsolete versions, clean
the affected package for the release target and rerun the script. For example:
`./scripts/cargo_with_vendor.sh clean --release --target aarch64-apple-darwin -p distance-oracle`.

This audits Rust packages compiled for the selected native executables and their
build tools. It does not prove that vulnerable functions are reachable, audit
system libraries or bundled C code, or separately inventory the nested Wasm
build or the runtime bytes already stored in the raw chain spec. Results apply
to the selected platform and feature set.

### Manual Linux audit in CI

Run `audit_production_linux` from any pipeline created by the repository's
workflow rules. It is optional and starts independently of the other jobs.
It prepares the SDK, downloads the G1 genesis from the upstream `g1-1100`
release, and generates G1 metadata with a temporary local node.
It then builds and audits both production executables for
`x86_64-unknown-linux-gnu` and runs `cargo audit` on the complete lockfile.

The job uses the same advisory exceptions as local audits and fetches the current
RustSec database on each run. A rejected dependency makes the job fail, with
`allow_failure: true` keeping this on-demand check from blocking the pipeline.
Reports in `target/audit/` are retained as artifacts for two weeks, including
failed runs. The Linux ARM64 build is not covered by this job.
