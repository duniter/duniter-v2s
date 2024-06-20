# Duniter v2s

🆙 A rewriting of [Duniter v1](https://duniter.org) in the [Substrate](https://www.substrate.io/) framework.

⚠️ Duniter-v2s is under active development.

🚧 A test network called "ĞDev" is deployed, allowing to test wallets and indexers.

<div align="center">
    <img alt="logov2" src="https://duniter.fr/img/duniterv2.svg" width="128" height="128"/>
</div>

## Documentation TOC

- [README](./README.md)
  - [Use](#use)
  - [Test](#test)
  - [Contribute](#contribute)
  - [Structure](#project-structure)
- [docker](./docker/) docker-related documentation
- [docs](./docs/)
  - [api](./docs/api/)
    - [manual](./docs/api/manual.md)
    - [runtime-calls](./docs/api/runtime-calls.md) the calls you can submit through the RPC API
    - [runtime-errors](./docs/api/runtime-errors.md) the errors you can get submitting a call
    - [runtime-events](./docs/api/runtime-events.md) the events you can get submitting a call
  - [dev](./docs/dev/)
    - [beginner-walkthrough](./docs/dev/beginner-walkthrough.md)
    - [git-conventions](./docs/dev/git-conventions.md)
    - [launch-a-live-network](./docs/dev/launch-a-live-network.md)
    - [setup](./docs/dev/setup.md)
    - [compilation features](./docs/dev/compilation.md)
    - [verify-runtime-code](./docs/dev/verify-runtime-code.md)
    - [weights-benchmarking](./docs/dev/weights-benchmarking.md)
    - [upgrade-substrate](./docs/dev/upgrade-substrate.md)
  - [test](./docs/test/)
    - [replay-block](./docs/test/replay-block.md)
  - [user](./docs/user/)
    - [autocompletion](./docs/user/autocompletion.md)
    - [mirror](./docs/user/mirror.md) deploy a permanent ǦDev mirror node
    - [smith](./docs/user/smith.md) deploy a permanent ǦDev validator node
    - [debian installation](./docs/user/installation_debian.md)
  - [packaging](./docs/packaging/)
    - [build-for-arm](./docs/packaging/build-for-arm.md)
    - [build-debian](./docs/packaging/build-deb.md) build a native Debian package
- [end2end-tests](./end2end-tests/) automated end to end tests written with cucumber
- [live-tests](./live-tests/) sanity checks to test the storage of a live chain

## Use

### Join ĞDev network

The easiest way is to use the docker image.

Minimal command to deploy a **temporary** mirror peer:

```docker
docker run -it -p9944:9944 -e DUNITER_CHAIN_NAME=gdev duniter/duniter-v2s:v0.4.0 --tmp --execution=Wasm
```

To go further, read [How to deploy a permanent mirror node on ĞDev network](./docs/user/rpc.md).

### Create your local blockchain

It can be useful to deploy your local blockchain, for instance to have a controlled environment to develop/test an application that interacts with the blockchain.

```docker
docker run -it -p9944:9944 duniter/duniter-v2s:v0.4.0 --tmp
```

Or use the `docker-compose.yml` at the root of this repository.

#### Control when your local blockchain should produce blocks

By default, your local blockchain produces a new block every 6 seconds, which is not practical in some cases.

You can decide when to produce blocks with the cli option `--sealing` which has two modes:

- `--sealing=instant`: produce a block immediately upon receiving a transaction into the transaction pool
- `--sealing=manual`: produce a block upon receiving an RPC request (method `engine_createBlock`).

### Autocompletion

See [autocompletion](./docs/user/autocompletion.md).

## Test

### Test a specific commit

At each commit on master, an image with the tag `debug-sha-********` is published, where `********`
corresponds to the first 8 hash characters of the commit.

Usage:

```docker
docker run -it -p9944:9944 --name duniter-v2s duniter/duniter-v2s:debug-sha-b836f1a6
```

Then open `https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944` in a browser.

Enable detailed logging:

```docker
docker run -it -p9944:9944 --name duniter-v2s \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=1 \
  -lruntime=debug \
  duniter/duniter-v2s:debug-sha-b836f1a6
```

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
cargo run -- --dev --tmp
```

This will deploy a local blockchain with test accounts (Alice, Bob, etc) in the genesis.

## Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/debug/duniter --dev --tmp
```

Then open `https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944` in a browser.

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/debug/duniter -lruntime=debug --dev
```

## Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).

### Purge previous local testnet

```
./target/debug/duniter purge-chain --base-path /tmp/alice --chain local
./target/debug/duniter purge-chain --base-path /tmp/bob --chain local

```

### Start Alice's node

```bash
./target/debug/duniter \
  --base-path /tmp/alice \
  --chain local \
  --alice \
  --port 30333 \
  --rpc-port 9944 \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --validator
```

### Start Bob's node

```bash
./target/debug/duniter \
  --base-path /tmp/bob \
  --chain local \
  --bob \
  --port 30334 \
  --rpc-port 9945 \
  --validator \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

## Project Structure

A Substrate project such as this consists of a number of components that are spread across a few
directories.

### Node

A blockchain node is an application that allows users to participate in a blockchain network.
Substrate-based blockchain nodes expose a number of capabilities:

- Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking stack to allow the
  nodes in the network to communicate with one another.
- Consensus: Blockchains must have a way to come to
  [consensus](https://substrate.dev/docs/en/knowledgebase/advanced/consensus) on the state of the
  network. Substrate makes it possible to supply custom consensus engines and also ships with
  several consensus mechanisms that have been built on top of
  [Web3 Foundation research](https://research.web3.foundation/en/latest/polkadot/NPoS/index.html).
- RPC Server: A remote procedure call (RPC) server is used to interact with Substrate nodes.

There are several files in the `node` directory - take special note of the following:

- [`chain_spec.rs`](./node/src/chain_spec.rs): A
  [chain specification](https://substrate.dev/docs/en/knowledgebase/integrate/chain-spec) is a
  source code file that defines a Substrate chain's initial (genesis) state. Chain specifications
  are useful for development and testing, and critical when architecting the launch of a
  production chain. Take note of the `development_chain_spec` and `testnet_genesis` functions, which
  are used to define the genesis state for the local development chain configuration. These
  functions identify some
  [well-known accounts](https://substrate.dev/docs/en/knowledgebase/integrate/subkey#well-known-keys)
  and use them to configure the blockchain's initial state.
- [`service.rs`](./node/src/service.rs): This file defines the node implementation. Take note of
  the libraries that this file imports and the names of the functions it invokes. In particular,
  there are references to consensus-related topics, such as the
  [longest chain rule](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#longest-chain-rule),
  the [Babe](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#babe) block authoring
  mechanism and the
  [GRANDPA](https://substrate.dev/docs/en/knowledgebase/advanced/consensus#grandpa) finality
  gadget.

After the node has been [built](#build), refer to the embedded documentation to learn more about the
capabilities and configuration parameters that it exposes:

```shell
./target/debug/duniter --help
```

### Runtime

In Substrate, the terms
"[runtime](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#runtime)" and
"[state transition function](https://substrate.dev/docs/en/knowledgebase/getting-started/glossary#stf-state-transition-function)"
are analogous - they refer to the core logic of the blockchain that is responsible for validating
blocks and executing the state changes they define. The Substrate project in this repository uses
the [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) framework to construct a
blockchain runtime. FRAME allows runtime developers to declare domain-specific logic in modules
called "pallets". At the heart of FRAME is a helpful
[macro language](https://substrate.dev/docs/en/knowledgebase/runtime/macros) that makes it easy to
create pallets and flexibly compose them to create blockchains that can address
[a variety of needs](https://www.substrate.io/substrate-users/).

Review the [FRAME runtime implementation](./runtime/src/lib.rs) included in this template and note
the following:

- This file configures several pallets to include in the runtime. Each pallet configuration is
  defined by a code block that begins with `impl $PALLET_NAME::Config for Runtime`.
- The pallets are composed into a single runtime by way of the
  [`construct_runtime!`](https://crates.parity.io/frame_support/macro.construct_runtime.html)
  macro, which is part of the core
  [FRAME Support](https://substrate.dev/docs/en/knowledgebase/runtime/frame#support-library)
  library.

### Pallets

The runtime in this project is constructed using many FRAME pallets that ship with the
[core Substrate repository](https://github.com/paritytech/substrate/tree/master/frame) and a
template pallet that is [defined in the `pallets`](./pallets/template/src/lib.rs) directory.

A FRAME pallet is compromised of a number of blockchain primitives:

- Storage: FRAME defines a rich set of powerful
  [storage abstractions](https://substrate.dev/docs/en/knowledgebase/runtime/storage) that makes
  it easy to use Substrate's efficient key-value database to manage the evolving state of a
  blockchain.
- Dispatchables: FRAME pallets define special types of functions that can be invoked (dispatched)
  from outside of the runtime in order to update its state.
- Events: Substrate uses [events](https://substrate.dev/docs/en/knowledgebase/runtime/events) to
  notify users of important changes in the runtime.
- Errors: When a dispatchable fails, it returns an error.
- Config: The `Config` configuration interface is used to define the types and parameters upon
  which a FRAME pallet depends.

## License

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
