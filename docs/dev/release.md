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
