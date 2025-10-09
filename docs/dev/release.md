# Releasing Duniter

This document describes the steps required for the different releasing processes related to a Duniter based network:

* network release (boostrap a new network)
* client release (duniter-v2s binary on bootstrap or update)
* runtime release (WASM runtime on bootstrap or update)

## Prerequisites

* Linux machine (Ubuntu 22.04 LTS tested)
* [Docker](https://docs.docker.com/get-docker/)
* [Podman](https://podman.io/getting-started/installation)
* [jq](https://stedolan.github.io/jq/download/)
* [Rustup](https://rustup.rs/)

Also, to publish releases on GitLab and DockerHub, you will need:

* a GitLab token (created in your GitLab profile, with full scope to avoid errors)
* the credentials of the [DockerHub duniter organization](https://hub.docker.com/u/duniter) (a.k.a.  ̀DUNITERTEAM_PASSWD` variable in the CI)

## Artifacts

Most artifacts are stored in the `release/` folder. You may want to clean it up before starting a new release:

```bash
rm -rf release/*
```

## Network release

A network release is required to bootstrap a new Duniter based network (a.k.a. currency).

This step will create a Genesis Data file (a dump of current Ĝ1 data) as well as the initial runtime for the new network,
and publish them as a GitLab release.

The successive Client releases for this network will refer to this network release to fetch the genesis specs.

### Prerequisites

You must know:

* the network name (e.g. `gdev`) refered as `<network>` in the following
* the initial runtime version (e.g. `1000`) refered as `<runtime-initial-version>` in the following

You must have:

* created a Git branch named `network/<network>-<runtime-initial-version>` (e.g. `network/gdev-1000`)
* edited the file `resources/<network>.yml` with the network parameters (commitee members, token symbol, etc.)
* set the `GITLAB_TOKEN` environment variable with your GitLab token

### Commands

To create the network release, run:

```bash
cargo xtask network-g1-data
cargo xtask network-build-runtime
cargo xtask network-build-specs
cargo xtask network-create-release <network>-<runtime-initial-version> network/<network>-<runtime-initial-version>
```

Example : 

```bash
export GITLAB_TOKEN=your_token_here
cargo xtask network-g1-data
cargo xtask network-build-runtime
cargo xtask network-build-specs
cargo xtask network-create-release gdev-1000 network/gdev-1000
```

### Notes

The Ĝ1 data is, by default, fetching the last dump of cgeek's node procuded every day at midnight.

## Client release

A client release is required to distribute a new version of the `duniter-v2s` binary.
It will be released under three flavors:

* DEB package for Debian x86 based systems
* RPM package for Fedora x86 based systems
* Docker image (both for amd64 and arm64 architectures)

### Prerequisites

You must know:

* the network name (e.g. `gdev`) refered as `<network>` in the following
* the client version (e.g. `0.11.1`) refered as `<client-version>` in the following

You must have:

* set the Client version in the `node/Cargo.toml` file (as `version = "<client-version>"`) on the network branch (not on `master`)
* a GitLab milestone named `client-<client-version>` (e.g. `client-0.11.1`)
* set the `GITLAB_TOKEN` environment variable with your GitLab token
* set the `DUNITERTEAM_PASSWD` environment variable set with the DockerHub password of the Duniter organization

### Commands

To create the client release, run:

```bash
cargo xtask client-build-raw-specs <network>-<runtime-initial-version>
cargo xtask client-build-deb <network>-<runtime-initial-version>
cargo xtask client-build-rpm <network>-<runtime-initial-version>
cargo xtask client-docker-deploy <network>-<runtime-initial-version>
cargo xtask client-create-release <network>-<runtime-initial-version> network/<network>-<runtime-initial-version>
```

Example : 

```bash
export GITLAB_TOKEN=your_token_here
export DUNITERTEAM_PASSWD=your_dockerhub_password_here
cargo xtask client-build-raw-specs gdev-1000
cargo xtask client-build-deb gdev-1000
cargo xtask client-build-rpm gdev-1000
cargo xtask client-docker-deploy gdev-1000
cargo xtask client-create-release gdev-1000 network/gdev-1000
```

## Runtime release

This step is required only for Runtime updates (e.g. new features, bug fixes, etc., but not for a network reboot).

### Prerequisites

You must know:

* the network name (e.g. `gdev`) refered as `<network>` in the following
* the new runtime version (e.g. `1100`) refered as `<new-runtime-version>` in the following

You must have:

* created a new branch named `runtime/<network>-<new-runtime-version>` (e.g. `runtime/gdev-1100`)
* set the new runtime version in the `runtime/<network>/src/lib.rs` file for the `spec_version` variable on that branch (not on `master`)
* a GitLab milestone named `runtime-<new-runtime-version>` (e.g. `runtime-1100`)
* set the `GITLAB_TOKEN` environment variable with your GitLab token

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
