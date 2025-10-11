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
- [Docker](https://docs.docker.com/get-docker/)
- [Podman](https://podman.io/getting-started/installation) with at least **16GB
  RAM** and **8 CPUs** allocated (use
  `podman machine set --memory 16384 --cpus 8`)
- [jq](https://stedolan.github.io/jq/download/)
- [Rustup](https://rustup.rs/)

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
cargo xtask network-g1-data
cargo xtask network-build-runtime <network>
cargo xtask network-build-specs <network>
cargo xtask network-create-release <network>-<runtime-initial-version> network/<network>-<runtime-initial-version>
```

Example :

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask network-g1-data
cargo xtask network-build-runtime gdev
cargo xtask network-build-specs gdev
cargo xtask network-create-release gdev-1000 network/gdev-1000
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
cargo xtask client-build-raw-specs <network>-<runtime-version>
cargo xtask client-create-release <network>-<runtime-version> network/<network>-<runtime-version>
```

Example:

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask client-build-raw-specs gdev-1000
cargo xtask client-create-release gdev-1000 network/gdev-1000
```

This creates the GitLab release with the basic specs files.

### Step 2: Build client packages and Docker images

You have **two options** to build and publish the client packages:

#### Option A: Automated CI builds (Recommended)

This method triggers the CI builds on GitLab runners, which will build all
packages (DEB, RPM, Docker) for both x86_64 and ARM64 architectures, then
automatically upload them to the release.

```bash
cargo xtask client-trigger-release-builds <network>-<runtime-version> network/<network>-<runtime-version> <release-tag>
```

The `<release-tag>` is the release name created in Step 1, in the format:
`<network>-<runtime-version>-<client-version>` (e.g., `gdev-1000-0.11.1`).

Example:

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask client-trigger-release-builds gdev-1000 network/gdev-1000 gdev-1000-0.11.1
```

This command will:

- Trigger a CI pipeline on the specified branch
- Start all 6 release jobs (Debian/RPM × ARM/x64 + Docker × ARM/x64)
- Monitor their execution
- Download artifacts from successful jobs
- Upload .deb and .rpm files to the release

**Note:** The Docker images will be pushed to DockerHub automatically by the CI
jobs. The `DUNITERTEAM_PASSWD` variable must be set in the GitLab CI/CD
variables.

#### Option B: Manual local builds

If you prefer to build packages locally (e.g., for testing or if CI is
unavailable), you can run these commands:

```bash
cargo xtask client-build-deb <network>-<runtime-version>
cargo xtask client-build-rpm <network>-<runtime-version>
cargo xtask client-docker-deploy <network>-<runtime-version>
```

Example:

```bash
export GITLAB_TOKEN=your_token_here
export DUNITERTEAM_PASSWD=your_dockerhub_password_here
cargo xtask client-build-deb gdev-1000
cargo xtask client-build-rpm gdev-1000
cargo xtask client-docker-deploy gdev-1000
```

**Note:** Manual builds will only create packages for your local architecture.
To build for multiple architectures, you need to use cross-compilation tools or
run the commands on different machines.

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
cargo xtask runtime-build <network>
cargo xtask runtime-create-release <network>> runtime/<network>-<new-runtime-version>
```

Example :

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask runtime-build gdev
cargo xtask runtime-create-release gdev runtime/gdev-1100
```
