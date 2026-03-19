# Releasing Duniter

This document describes the supported release workflows for Duniter:

1. **New network release**: bootstrap a brand new network with a new genesis.
2. **Client release on an existing network**: ship a new `duniter` binary without changing genesis.
3. **Runtime release on an existing network**: ship a new WASM runtime for an already running network.

The three flows do not use the same tooling anymore:

| Release type | Supported tooling | Notes |
| --- | --- | --- |
| New network (new genesis) | `scripts/launch-network.sh`, `cargo xtask`, GitLab CI | The script orchestrates the full bootstrap flow. |
| Client release on existing network | GitLab CI only | Do not use the old `cargo xtask release client ...` upgrade flow for this case. |
| Runtime release on existing network | `cargo xtask` only | This publishes the runtime artifacts; the upgrade is then submitted on-chain. |

## Common conventions

### Network release tag

Network releases use the following tag format:

```text
<network>-<genesis-runtime-version>
```

Examples:

- `gdev-1000`
- `g1-1100`

### Client release tag

Client releases use the following tag format:

```text
<network>-<genesis-runtime-version>-<client-version>
```

Examples:

- `gdev-1000-0.11.1`
- `g1-1100-2.0.1`

Important: the middle component is always the **genesis runtime version** of
that network. It does not change after runtime upgrades.

Example: if `g1` was bootstrapped with runtime `1100`, the next client release
must still be tagged `g1-1100-2.1.0`, even if the latest runtime release is now
`g1-1200`.

### Local artifacts

Most locally generated artifacts are stored in `release/`. Before starting a
fresh release flow, it is usually safer to clean it:

```bash
rm -rf release/*
```

## 1. New network release (new genesis)

Use this flow when you bootstrap a new network from scratch. This is the only
release flow that combines:

- local `cargo xtask` commands to build genesis artifacts
- GitLab API calls to publish releases
- GitLab CI jobs to build packages and Docker images for the bootstrap client

### Supported entrypoint

Use `scripts/launch-network.sh`.

Example:

```bash
./scripts/launch-network.sh --runtime g1
```

Useful options:

```bash
./scripts/launch-network.sh --runtime g1 --resume-from 6
./scripts/launch-network.sh --runtime g1 --skip-build
```

### What the script does

For a new network, `scripts/launch-network.sh` automates the supported flow:

1. validate the local prerequisites and release credentials
2. run the network xtask steps:
   - `cargo xtask release network g1-data`
   - `cargo xtask release network build-runtime <network>`
   - `cargo xtask release network build-specs <network>`
   - `cargo xtask release network create <network>-<genesis-runtime-version> network/<network>-<genesis-runtime-version>`
3. prepare the bootstrap client release for the same network
4. trigger the GitLab CI release jobs for packages and Docker images

### Required preparation

Before running the script, make sure you already have:

- a branch named `network/<network>-<genesis-runtime-version>`
- the network parameters in `resources/<network>.yaml`
- the client specs in `node/specs/<network>_client-specs.yaml`
- the client version updated in `node/Cargo.toml`
- `GITLAB_TOKEN` exported locally
- `DUNITERTEAM_PASSWD` exported locally

### Related documents

- `docs/dev/g1-production-launch.md`: step-by-step operational guide for the G1 launch
- `docs/dev/g1-launch-checklist.md`: launch checklist and prerequisites

## 2. Client release on an existing network

Use this flow when you want to release a new node binary for a network that
already exists, without creating a new genesis.

This flow is now **GitLab CI only**.

Do not use the legacy `cargo xtask release client build-raw-specs`,
`trigger-builds`, or `finalize` flow for this case.

### Prerequisites

- the client version is updated in `node/Cargo.toml`
- the branch or commit you want to release is already pushed to GitLab
- the release tag will be created as a **protected tag**, otherwise the release
  jobs will not run
- if you want the GitLab release notes to be populated automatically, create the
  GitLab milestone `client-<client-version>` and attach the relevant merged MRs
  to it

Treat any change to the released node binary as a client release and bump the
client version accordingly. In particular, changes to the embedded raw chainspec
(for example bootnodes, because the release build embeds the raw spec in the
binary) require a new client version for the network being released.

### Release steps

#### Step 1: prepare the version

Update `node/Cargo.toml` on the branch you want to release:

```toml
version = "<client-version>"
```

#### Step 2: create and push the release tag

Create a tag using the genesis runtime version of the network:

```bash
git tag <network>-<genesis-runtime-version>-<client-version>
git push origin <network>-<genesis-runtime-version>-<client-version>
```

Example:

```bash
git tag g1-1100-2.0.1
git push origin g1-1100-2.0.1
```

#### Step 3: let GitLab CI build and publish the release

Pushing the protected tag triggers the release pipeline automatically. The CI
pipeline will:

- build Debian packages for amd64 and arm64
- build RPM packages for x86_64 and aarch64
- build Docker images for amd64 and arm64
- publish the multi-architecture Docker manifest
- create or update the GitLab release page and attach the package assets

The GitLab release page is created by CI through
`scripts/create_gitlab_client_release.sh`.

### GRANDPA hard fork after forced finality recovery

If a network has recovered finality with `grandpa.noteStalled`, a later client
release may need a network-specific `AuthoritySetHardFork` in
`node/src/service.rs` so that new nodes can sync correctly.

This must be configured carefully:

- Add the hard fork only for the live chain spec id of the affected network.
- Model only the single forced authority-set jump that was actually finalized.
- Do not encode all intermediate on-chain `Grandpa.NewAuthorities` events that
  happened while finality was stalled.

For a forced recovery, the hard fork entry must be derived from the
`ConsensusLog::ForcedChange`, not from the later `Grandpa.NewAuthorities`
application block:

- `block`: use the block that contains the `ForcedChange` digest log.
- `set_id`: use the effective GRANDPA offchain set id after recovery.
- `authorities`: use the authority list of the recovered live set.
- `last_finalized`: set it to `Some(median_last_finalized)` from the
  `ForcedChange` digest.

Do not anchor the hard fork on the later `Grandpa.NewAuthorities` block with
`last_finalized: Some(...)`. In the SDK, `AuthoritySetHardFork` is converted to
a pending forced change with `delay = 0`. If you anchor it on the application
block, the node will see:

- the original on-chain forced change from the earlier digest
- plus the local hard fork forced change

This creates two simultaneous pending forced changes and sync fails with:

```text
Multiple pending forced authority set changes are not allowed.
```

Recommended procedure to derive the correct config:

1. Identify the `grandpa.noteStalled` recovery window.
2. Query the chain around that point and find the header digest containing
   `ConsensusLog::ForcedChange`.
3. Decode from that digest:
   - the `median_last_finalized`
   - the delay
   - the recovered authority list
4. Use the block that contains that digest as the hard fork `block`.
5. Use the recovered live GRANDPA offchain `set_id`, not the on-chain runtime
   `CurrentSetId` if they diverged during the stall.
6. Gate the config to the live network only, for example by checking
   `config.chain_spec.id()`.
7. Verify with a fresh sync from genesis or fast sync before releasing.

Operational notes:

- Existing nodes that already synced without the hard fork may need a resync to
  rebuild their local GRANDPA metadata with the new interpretation.
- To validate `warp sync WithProvider`, at least one provider node must run the
  new binary and be resynced from scratch before using it as the provider for a
  second fresh node.
- In Duniter, `warp sync WithProvider` also requires `--no-checkpoint`.
  Otherwise the node uses `WithTarget` by default when a checkpoint is embedded
  or provided.

Concrete `gtest` example:

- `grandpa.noteStalled` was executed at block `#2187746`.
- The actual forced change was announced later by a `ConsensusLog::ForcedChange`
  digest in block `#2188261`, not at the `noteStalled` block itself.
- The forced change carried:
  - `median_last_finalized = 2_123_891`
  - `delay = 600`
- The recovered live GRANDPA set after the recovery used offchain `set_id = 115`
  with the following six authorities:
  - `0x10c3f8cd9768029be7e32f125031d8540c1e8d9d8af54ab104fabf12e7291e8a`
  - `0x8760a45a6b359b30ddc3aa9a160f41e4d6d3a72a5eacef7cfaf00285757cc9b1`
  - `0xfd823b99be9106836fd685c1a8716b108aff80bf5220114498f224d29d06a95f`
  - `0x92eea3f0194c2c53e07015bb642f2ea00196e48b730b964a7dacbdf465119951`
  - `0xc4be7d48526f5bfa4c3427fca551abdcf45e90e9a750bfaccac1b60635218e67`
  - `0x40bebbf11d0cfad19a65fd0756994d16c53d0bed57067cbdf0c764df5853704b`

The corresponding hard fork entry is therefore anchored on block `#2188261`
with hash `0x275b7296dab9ab0d925b4a835f919f0272cb68dc5ee44a658cceb1f84bc4d02f`,
not on the later `Grandpa.NewAuthorities` application block `#2188861`.

Using `#2188861` with `last_finalized: Some(2_123_891)` causes sync failures
because the node sees both:

- the original pending forced change from the on-chain digest at `#2188261`
- the local hard fork forced change added at `#2188861`

which triggers:

```text
Multiple pending forced authority set changes are not allowed.
```

### Important note about image tags

The Git tag keeps the **genesis runtime version** in its middle component, but
the Docker image tag uses the **current runtime spec version** from the source
tree being released.

That means a client release can legitimately use:

- Git tag: `g1-1100-2.1.0`
- Docker tag: `1200-2.1.0`

if `1200` is the current runtime `spec_version` in the release branch.

## 3. Runtime release on an existing network

Use this flow when the on-chain runtime changes on an already running network.
This is the flow for runtime upgrades, not for new genesis creation and not for
client-only packaging.

This flow uses `cargo xtask` only.

### Prerequisites

You must have:

- a branch named `runtime/<network>-<new-runtime-version>`
- the new `spec_version` already bumped in `runtime/<network>/src/lib.rs`
- a GitLab milestone named `runtime-<network>-<new-runtime-version>`
- `GITLAB_TOKEN` exported locally

### Commands

```bash
cargo xtask release runtime build <network>
cargo xtask release runtime create <network> runtime/<network>-<new-runtime-version>
```

Example:

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask release runtime build gdev
cargo xtask release runtime create gdev runtime/gdev-1100
```

This produces and publishes the runtime artifacts on GitLab. It does **not**
perform the on-chain upgrade by itself.

### On-chain submission

Once the release is published, submit the runtime upgrade through governance.

See:

- `docs/dev/runtime-upgrade.md` for the full Technical Committee procedure
- `docs/dev/verify-runtime-code.md` for the runtime hash verification flow
