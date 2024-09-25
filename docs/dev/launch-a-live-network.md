# How to launch a live network

Launching a new live network is more difficult than spawning a local blockchain. Follow this process if you know what you are doing and you already experimented a bit with local blockchains. Part of this process is automated with Rust scripts, including interaction with GitLab's GraphQL API. Do not hesitate to improve and complete it (see TODOs inside `xtask/**/*.rs` files).

## Requirements

In order to build in a standardized environment, you need Docker.

- see docker docs to [install docker](https://docs.docker.com/engine/install/)
- make sure you can run docker as non-root user with `docker info` or so

## Preparation

When launching a new network, you're likely to use a new runtime. See how to [release a new runtime](./release-new-runtime.md).

### Inject runtime in chainspec

~~ĞDev runtime is automatically embeded in the raw chainspec with the `include_bytes!` macro. An other way~~ to inject the runtime is to use "inject-runtime-code" xtask:

```bash
cargo xtask inject-runtime-code --runtime runtime/gdev/target/srtool/release/wbuild/gdev-runtime/gdev_runtime.compact.compressed.wasm --raw-spec resources/gdev-raw.json
```

## Bootstraping

### Choose the currency type

For now, only `gdev` is supported.

In the commands that will be indicated afterwards, you will have to replace `CURRENCY` by the
currency type you have chosen.

### Choose the docker image

Choose the docker image that contains the version of the code that you want to use.

In the commands that will be indicated afterwards, you will have to replace `TAG` by the tag of the
docker image that you have chosen (example : runtime-400).

### Generate the session keys of genesis authority

Generate a random secret phrase:

```bash
$ docker run --rm duniter/duniter-v2s:TAG -- key generate
Secret phrase:       noble stay fury mean poverty delay stadium organ evil east vague can
  Secret seed:       0xb39c31fb10c5080721738880c2ea45412cb3df33df022bf8d9a51483b3a9b7a6
  Public key (hex):  0x90a0c2866034db9d05f8193a95fe5af8d5e12ab295a501c17c95cdbeaf226d62
  Account ID:        0x90a0c2866034db9d05f8193a95fe5af8d5e12ab295a501c17c95cdbeaf226d62
  Public key (SS58): 5FLLWRsxdLKfXH9VQH6Yv73hN1oc9KoFkZ5LEHEz1uTR1Qt3
  SS58 Address:      5FLLWRsxdLKfXH9VQH6Yv73hN1oc9KoFkZ5LEHEz1uTR1Qt3
```

Keep this secret phrase **carefully**, it will be used **several** times later.

Then, generate the session keys:

```bash
$ docker run --rm duniter/duniter-v2s:TAG -- key generate-session-keys --chain CURRENCY_local --suri "<your secret phrase>"
Session Keys: 0x87189d723e1b2826c243bc433c718ac26ba60526932216a09102a254d54462b890a0c2866034db9d05f8193a95fe5af8d5e12ab295a501c17c95cdbeaf226d6290a0c2866034db9d05f8193a95fe5af8d5e12ab295a501c17c95cdbeaf226d6290a0c2866034db9d05f8193a95fe5af8d5e12ab295a501c17c95cdbeaf226d62
```

### Paste sessions keys in the genesis configuration file

An example of genesis configuration file: `resources/gdev.json`. Paste your session keys in your `smith` identity with key `session_keys`.

### Generate raw spec

```docker
docker run -v $HOME/dev/duniter-v2s/resources:/var/lib/duniter/resources -e DUNITER_GENESIS_CONFIG=/var/lib/duniter/resources/gdev.json --rm duniter/duniter-v2s:TAG -- build-spec -lerror --chain=gdev_live --raw > name-raw.json
```

```bash
./scripts/gen-live-network-raw-spec.sh CURRENCY "<path/to/your/genesis/config/file>"
```

This builds the specs using debug version of Duniter.

### Generate the docker compose and prepare nodes keys

```bash
./scripts/create-live-network.sh "<your secret phrase>" CURRENCY "<path/to/dist/folder>"
```

The new distribution folder can be copied to a server

```bash
scp -r "<path/to/dist/folder>" <server>:/remote/dist/path
```

then on the server, launch the compose file from the the distribution folder's root:

```bash
ssh <server>
cd "<path/to/dist/folder>"
docker compose up -d
```

This is the first node of the new live network.

## Finalization

The following steps should be completed once you are satisfied with the new live network.

### Rotate session keys

You should rotate session keys for more secured keys produced on the server (the one you used before are still in your develop machine bash history and clipboard).

### Publish image

With these new session keys in the chainspec and the runtime build with srtool, you can release the new runtime again with:

```bash
cargo xtask release-runtime 400
```

### Tell the other smith

Once you completed all these steps, the other smith can pull the docker image with a genesis containing your bootnode with the correct session keys. They can base their `docker-compose.yml` on the `duniter-validator` template.
