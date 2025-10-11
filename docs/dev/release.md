# Releasing Duniter

This document describes the steps required for the different releasing processes
related to a Duniter based network:

- **network** release (boostrap a new network)
- **client** release (duniter-v2s binary on bootstrap or update)
- **runtime** release (WASM runtime on bootstrap or update)

## Philosophy

The release process will produce artifacts according to your **local basecode**
(i.e. the code on your machine), not the code on GitLab.

## Prerequisites

- Linux machine (Ubuntu 22.04 LTS tested)
- [Docker](https://docs.docker.com/get-docker/) **OR**
  [Podman](https://podman.io/getting-started/installation) with at least **16GB
  RAM** and **8 CPUs** allocated (use
  `podman machine set --memory 16384 --cpus 8`)
- [jq](https://stedolan.github.io/jq/download/)
- [Rustup](https://rustup.rs/)

**Note:** The build system automatically detects whether Docker or Podman is
available and uses the appropriate tool.

Also, to publish releases on GitLab and DockerHub, you will need:

- a GitLab token (created in your GitLab profile, with full scope to avoid
  errors)
- the credentials of the
  [DockerHub duniter organization](https://hub.docker.com/u/duniter) (a.k.a.
  ̀DUNITERTEAM_PASSWD` variable in the CI)

## Artifacts

Most artifacts are stored in the `release/` folder. You may want to clean it up
before starting a new release:

```bash
rm -rf release/*
```

## Network release

A network release is required to bootstrap a new Duniter based network (a.k.a.
currency).

This step will create a Genesis Data file (a dump of current Ĝ1 data) as well as
the initial runtime for the new network, and publish them as a GitLab release.

The successive Client releases for this network will refer to this network
release to fetch the genesis specs.

### Prerequisites

You must know:

- the network name (e.g. `gdev`) refered as `<network>` in the following
- the initial runtime version (e.g. `1000`) refered as
  `<runtime-initial-version>` in the following

You must have:

- created a Git branch named `network/<network>-<runtime-initial-version>` (e.g.
  `network/gdev-1000`)
- edited the file `resources/<network>.yml` with the network parameters
  (commitee members, token symbol, etc.)
- set the `GITLAB_TOKEN` environment variable with your GitLab token

### Commands

To create the network release, run:

```bash
cargo xtask release network g1-data
cargo xtask release network build-runtime <network>
cargo xtask release network build-specs <network>
cargo xtask release network create <network>-<runtime-initial-version> network/<network>-<runtime-initial-version>
```

Example :

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask release network g1-data
cargo xtask release network build-runtime gdev
cargo xtask release network build-specs gdev
cargo xtask release network create gdev-1000 network/gdev-1000
```

### Notes

The Ĝ1 data is, by default, fetching the last dump of cgeek's node procuded
every day at midnight.

## Client release

A client release is required to distribute a new version of the `duniter-v2s`
binary. It will be released under three flavors:

- DEB package for Debian (x86_64 and ARM64 architectures)
- RPM package for Fedora (x86_64 and ARM64 architectures)
- Docker image (amd64 and arm64 architectures)

### Prerequisites

You must know:

- the network name (e.g. `gdev`) refered as `<network>` in the following
- the runtime version (e.g. `1000`) refered as `<runtime-version>` in the
  following
- the client version (e.g. `0.11.1`) refered as `<client-version>` in the
  following

You must have:

- set the Client version in the `node/Cargo.toml` file (as
  `version = "<client-version>"`) on the network branch (not on `master`)
- edited the file `node/specs/<network>_client-specs.yaml` with the network
  parameters (commitee members, token symbol, etc.)
- a GitLab milestone named `client-<client-version>` (e.g. `client-0.11.1`)
- set the `GITLAB_TOKEN` environment variable with your GitLab token
- set the `DUNITERTEAM_PASSWD` environment variable with the DockerHub password
  of the Duniter organization (only for manual Docker deployment)

### Step 1: Build raw specs and create GitLab release

First, build the raw specs and create the GitLab release:

```bash
cargo xtask release client build-raw-specs <network>-<runtime-version>
cargo xtask release client create <network>-<runtime-version> network/<network>-<runtime-version>
```

Example:

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask release client build-raw-specs gdev-1000
cargo xtask release client create gdev-1000 network/gdev-1000
```

This creates the GitLab release with the specs files
(`<network>_client-specs.yaml` and `<network>-raw.json`).

**Note:** By default, this command only uploads the specs files. If you want to
also upload local DEB/RPM packages, use the `--upload-packages` flag. However,
it's recommended to use the automated CI builds (Step 2, Option A) instead.

### Step 2: Build client packages and Docker images

You have **two options** to build and publish the client packages:

#### Option A: Automated CI builds (Recommended)

This method triggers the CI builds on GitLab runners, which will build all
packages (DEB, RPM, Docker) for both x86_64 and ARM64 architectures, then
automatically upload them to the release.

```bash
cargo xtask release client trigger-builds <network>-<runtime-version> network/<network>-<runtime-version>
```

The release tag is automatically computed from the network name and client
version in `node/Cargo.toml`. You can also specify it manually with
`--release-tag` if needed.

Example:

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask release client trigger-builds gdev-1000 network/gdev-1000
```

This command will:

- Trigger a CI pipeline on the specified branch
- Start 7 release jobs:
  - 4 package builds (Debian/RPM × ARM/x64)
  - 2 Docker builds (one per architecture on dedicated runners)
  - 1 manifest creation job (combines both Docker images into a single tag)
- Monitor their execution
- Download artifacts from successful jobs (.deb and .rpm)
- Upload .deb and .rpm files to the release

**Docker images:** The CI builds Docker images in parallel on ARM and x64
runners, then creates a **single multi-arch manifest** (e.g.,
`duniter/duniter-v2s-gtest:1100-0.12.0`) that works automatically on both
architectures. The `DUNITERTEAM_PASSWD` variable must be set in the GitLab CI/CD
variables.

#### Option B: Manual local builds

If you prefer to build packages locally (e.g., for testing or if CI is
unavailable), you can run these commands:

```bash
cargo xtask release client build-deb <network>-<runtime-version>
cargo xtask release client build-rpm <network>-<runtime-version>
```

Example:

```bash
cargo xtask release client build-deb gdev-1000
cargo xtask release client build-rpm gdev-1000
```

Then upload them to the release:

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask release client create gdev-1000 network/gdev-1000 --upload-packages
```

**Note:** Manual builds will only create packages for your local architecture.
To build for multiple architectures, you need to use cross-compilation tools or
run the commands on different machines.

### Step 3: Build and push Docker images (Optional - for local testing)

Docker images are normally built automatically by the CI (Option A). However,
you can also build them locally for testing:

```bash
export DUNITERTEAM_PASSWD=your_dockerhub_password_here
cargo xtask release client docker <network>-<runtime-version>
```

Example:

```bash
export DUNITERTEAM_PASSWD=your_dockerhub_password_here
cargo xtask release client docker gdev-1000
```

This builds a multi-architecture image (amd64 + arm64) and pushes it to
DockerHub. The build system automatically detects whether to use Docker or
Podman.

**Note:** The `--arch` flag can be used to build for a specific architecture
only (e.g., `--arch amd64`), which pushes an image with the architecture in the
tag (e.g., `1100-0.12.0-amd64`). This is mainly for CI use or testing.

## Runtime release

This step is required only for Runtime updates (e.g. new features, bug fixes,
etc., but not for a network reboot).

### Prerequisites

You must know:

- the network name (e.g. `gdev`) refered as `<network>` in the following
- the new runtime version (e.g. `1100`) refered as `<new-runtime-version>` in
  the following

You must have:

- created a new branch named `runtime/<network>-<new-runtime-version>` (e.g.
  `runtime/gdev-1100`)
- set the new runtime version in the `runtime/<network>/src/lib.rs` file for the
  `spec_version` variable on that branch (not on `master`)
- a GitLab milestone named `runtime-<new-runtime-version>` (e.g. `runtime-1100`)
- set the `GITLAB_TOKEN` environment variable with your GitLab token

### Commands

To create the runtime release, run:

```bash
cargo xtask release runtime build <network>
cargo xtask release runtime create <network> runtime/<network>-<new-runtime-version>
```

Example :

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask release runtime build gdev
cargo xtask release runtime create gdev runtime/gdev-1100
```
