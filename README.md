# Duniter v2s

🆙 A rewriting of [Duniter v1](https://duniter.org) in the [Substrate](https://www.substrate.io/) framework.

⚠️ Duniter-v2s is under active development.

🚧 A test network called "ĞDev" is deployed, allowing to test wallets and indexers.

<div align="center">
    <img alt="logov2" src="https://duniter.fr/img/duniterv2.svg" width="128" height="128"/>
</div>

## Documentation

Multiple documentation sources are available depending on the level of detail you need.

- Full technical Rust doc (auto-generated with `cargo xtask gen-doc`) : https://doc-duniter-org.ipns.pagu.re/duniter/
- User and client developer doc (official website) : https://duniter.org/wiki/duniter-v2/
- Internal documentation (within git repository), see table of contents below : [./doc](./doc)

### Internal documentation TOC

- [README](./README.md) (this file)
  - [Use](#use)
  - [Contribute](#contribute)
  - [License](#license)
- [docs](./docs/) internal documentation
  - [api](./docs/api/) API
    - [manual](./docs/api/manual.md) manage account and identities
    - [runtime-calls](./docs/api/runtime-calls.md) the calls you can submit through the RPC API
    - [runtime-errors](./docs/api/runtime-errors.md) the errors you can get submitting a call
    - [runtime-events](./docs/api/runtime-events.md) the events you can get submitting a call
  - [dev](./docs/dev/) developer documentation
    - [beginner-walkthrough](./docs/dev/beginner-walkthrough.md)
    - [git-conventions](./docs/dev/git-conventions.md)
    - [pallet_conventions](./docs/dev/pallet_conventions.md)
    - [launch-a-live-network](./docs/dev/launch-a-live-network.md)
    - [setup](./docs/dev/setup.md)
    - [compilation features](./docs/dev/compilation.md)
    - [verify-runtime-code](./docs/dev/verify-runtime-code.md)
    - [weights-benchmarking](./docs/dev/weights-benchmarking.md)
    - [upgrade-substrate](./docs/dev/upgrade-substrate.md)
    - [replay-block](./docs/test/replay-block.md)
  - [user](./docs/user/) user documentation
    - [autocompletion](./docs/user/autocompletion.md)
    - [debian installation](./docs/user/installation_debian.md)
    - [distance](./docs/user/distance.md)
    - [fees](./docs/user/fees.md)
  - [packaging](./docs/packaging/) packaging
    - [build-for-arm](./docs/packaging/build-for-arm.md) build for ARM architecture
    - [build-debian](./docs/packaging/build-deb.md) build a native Debian package
- [docker](./docker/) docker-related documentation
- [end2end-tests](./end2end-tests/) automated end to end tests written with cucumber
- [live-tests](./live-tests/) sanity checks to test the storage of a live chain

## Use

### Join ĞDev network

The easiest way is to use the docker image.

Minimal command to deploy a temporary mirror peer:

```docker
docker run -it -p9944:9944 -e DUNITER_CHAIN_NAME=gdev duniter/duniter-v2s-gdev-800:latest
```

To go further, read [How to deploy a permanent mirror node on ĞDev network 🔗](https://duniter.org/wiki/duniter-v2/#run-a-mirror-node).

### Create your local blockchain

It can be useful to deploy your local blockchain, for instance to have a controlled environment to develop/test an application that interacts with the blockchain.

```docker
docker run -it -p9944:9944 duniter/duniter-v2s-gdev-800:latest
```

Or use the [`docker-compose.yml`](./docker-compose.yml) at the root of this repository.

#### Control when your local blockchain should produce blocks

By default, your local blockchain produces a new block every 6 seconds, which is not practical in some cases.

You can decide when to produce blocks with the cli option `--sealing` which has two modes:

- `--sealing=instant`: produce a block immediately upon receiving a transaction into the transaction pool
- `--sealing=manual`: produce a block upon receiving an RPC request (method `engine_createBlock`).

### Shell autocompletion

See [autocompletion](./docs/user/autocompletion.md) to generate shell autocompletion for duniter commands.

## Contribute

If you are beginner in Rust and need a well guided tutorial, follow the [beginner walkthrough](./docs/dev/beginner-walkthrough.md).

Before any contribution, please read carefully the [CONTRIBUTING](./CONTRIBUTING.md) file and our [git conventions](./docs/dev/git-conventions.md).

### Setup your dev environment

First, complete the [basic setup instructions](./docs/dev/setup.md).

### Build

NOTE: You must first follow the instructions in the [Setup](#setup-your-dev-environment) section.

Use the following command to build the node without launching it:

```sh
cargo build
```

### Run

Use Rust's native `cargo` command to build and launch the node:

```sh
cargo run -- --dev
```

This will deploy a local blockchain with test accounts (Alice, Bob, etc) in the genesis.
Open `https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944` to watch and interact with your node.

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/debug/duniter -lruntime=debug --dev
```

## License

See [LICENSE](./LICENSE)

```
CopyLeft 2021-2023 Axiom-Team

Some parts borrowed from Polkadot (Parity Technologies (UK) Ltd.)

Duniter-v2S is free software: you can redistribute it and/or modify
it under the terms of the **GNU Affero General Public License** as published by
the Free Software Foundation, **version 3** of the License.

Duniter-v2S is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with Duniter-v2S. If not, see <https://www.gnu.org/licenses/>.
```
