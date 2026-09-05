# Repository instructions

- Read `AGENTS.local.md` if present, then follow `CONTRIBUTING.md` and relevant docs.
- Preserve unrelated work.

## Validation

- Use the commands in `.gitlab-ci.yml`: prepare the SDK with `sdk_resolve` before running `fmt_and_clippy` and `tests_gdev` locally. Plain `cargo test` is insufficient.
- Run full validation for production-code changes only. Use focused checks for docs, tooling, and benchmarks.
- For node, RPC, networking, or end-to-end changes, also run `tests_cucumber`. Run `gtest` checks only to reproduce CI failures.
- For runtime or metadata changes, run `METADATA_CHAINS=gdev ./scripts/generate_all_metadata.sh`, with `subxt` installed.
- Measure realistic production load for per-block, consensus, networking, sink, or indexing changes. Bound work by the block, batch, or reorganization route.

## Releases

- Follow `docs/dev/release.md` for build and publication procedures.
- For binary releases, use updated `master`, update `Cargo.lock` with the version bump, commit as `v<version>`, and push. Start the release pipeline's manual prerequisites and verify its assets.
- For runtime releases, verify the artifact using `docs/dev/verify-runtime-code.md`. Then test the upgrade through governance on a local Chopsticks fork of the live G1 network, following `docs/dev/runtime-upgrade.md` on that fork only. Verify the new runtime version, migrations, and subsequent block execution. If all checks pass, report the results and stop. Never submit upgrade transactions to a live network.

## Commits and merge requests

- Write commit bodies as paragraphs with separate `-m` arguments. Check with `git log -1 --format='%B'`.
- Before creating an MR, check that the source branch has no open MR. Describe behavior, compatibility, and validation alongside the goal.
- Set release-note labels and the runtime milestone as required by `scripts/check_labels.sh`.
- Read failed CI traces before editing code. Fix MR metadata for `check_labels` failures. If prerequisites were skipped, start a fresh pipeline.
- A missing `duniter-polkadot-sdk.git` means a missing `sdk_resolve` artifact, not a Rust failure.
- Keep GitLab tokens in environment variables and authentication headers, never in URLs or output.
