# Beginner walkthrough

This is a beginner tutorial for those who do not have a previous experience with Rust ecosystem or need guidance to get familiar with Duniter v2s project. You'll need a development machine with an internet connection, at least **20 GB of free storage**, and **an hour or two** depending on your computing power.

This walkthrough is based on the following video (french), don't hesitate to record an english voiceover if you feel so.

[![preview](https://tube.p2p.legal/lazy-static/previews/654006dc-66c0-4e37-a32f-b7b5a1c13213.jpg)](https://tube.p2p.legal/w/n4TXxQ4SqxzpHPY4TNMXFu)

> video walkthrough on peertube https://tube.p2p.legal/w/n4TXxQ4SqxzpHPY4TNMXFu

## Requirements

If you are on a debian based system, you can install the required packages with:

```bash
sudo apt install cmake pkg-config libssl-dev git build-essential clang libclang-dev curl
```

Else, look at the corresponding section in the [system setup documentation](./setup.md).

Rust recommended installation method is through the rustup script that you can run with:

```bash
curl https://sh.rustup.rs -sSf | sh
```

If you reopen your terminal, it will give you access to the `rustup`,  `rustc` and `cargo` commands. You can then install the required Rust toolchains with:

```bash
rustup default stable
rustup update nightly
rustup update stable
rustup target add wasm32-unknown-unknown --toolchain nightly
```

This can take about **2 minutes**.

## Build project

After cloning wherever you want the `duniter-v2s` repo with:

```bash
git clone https://git.duniter.org/nodes/rust/duniter-v2s.git
```

you can go to the root folder and build the substrate client and default runtime with:

```bash
cargo build
```

This will take about **2 minutes** to download dependencies plus between 5 and **15 minutes** to build in debug mode depending on the power of your processor. At this point, you built the *substrate client* (a kind of "shell" in which lies the runtime) and the default *runtime* itself. You can run a local blockchain with:

```bash
cargo run -- --dev --tmp # here, --dev means --chain=dev which selects the gdev runtime
```

When you see the logs, the blockchain is running and you can connect to it with polkadotjs app: [https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944). You should see blocks being added every 6 seconds. You can use Alice, Bob, etc test accounts to submit extrinsics.

## Autocompletion

When using Duniter commands, you will benefit a lot from commands autocompletion. This can be achieved by following [autocompletion documentation](../user/autocompletion.md) for you shell. If you use bash the commands are:

```bash
# create local dir to store completion script
mkdir -p ~/.local/share/duniter
# export the bash completion file
cargo run -- completion --generator bash > ~/.local/share/duniter/completion.bash
# add the following line to your ~/.bashrc to automatically load completion on startup
[[ -f $HOME/.local/share/duniter/completion.bash ]] && source $HOME/.local/share/duniter/completion.bash
```

You will then benefit from completion using `<Tab>` key and `*`.

## End-to-end tests using cucumber

Cucumber end2end tests are a good way to dive in Duniter's business procedure. They work by spawning a local blockchain and submitting extrinsics to it. You can build and run the cucumber tests by running:

```bash
cargo cucumber
```

which should take about **4 minutes** to build and run the tests. A highly detailed documentation about the end2end tests is available [in the dedicated folder](../../end2end-tests/README.md), you will learn how to read and modify the tests.

## Get in touch with us

Wether you are stuck and need help or have sucessfully completed this tutorial, don't hesitate to get in touch with us on the Duniter forum! If you found this walkthrough useful, please 🙏 let us know on the [walkthrough topic](https://forum.duniter.org/t/contribuer-a-duniter-tutoriel-video/9770) on the forum 😊.