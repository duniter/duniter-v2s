# How to launch a live network

## 1. Choose the currency type

Ensure that the currency type you want have the requirements.

For now, only `gdev` is supported.

In the commands that will be indicated afterwards, you will have to replace `CURRENCY` by the
currency type you have chosen.

## 2. Choose the docker image

Choose or build the docker image that contains the version of the code thut you want to use.

In the commands that will be indicated afterwards, you will have to replace `TAG` by the tag of the
docker image that you have chosen.

## 3. Generate the session keys of genesis authority

Generate a random secret phrase:

```bash
$ docker run --rm -it --entrypoint duniter duniter/duniter-v2s:TAG key generate
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
$ docker run --rm -it --entrypoint duniter duniter/duniter-v2s:TAG key generate-session-keys --chain CURRENCY_local --suri "<your secret phrase>"
Session Keys: 0x87189d723e1b2826c243bc433c718ac26ba60526932216a09102a254d54462b890a0c2866034db9d05f8193a95fe5af8d5e12ab295a501c17c95cdbeaf226d6290a0c2866034db9d05f8193a95fe5af8d5e12ab295a501c17c95cdbeaf226d6290a0c2866034db9d05f8193a95fe5af8d5e12ab295a501c17c95cdbeaf226d62
```

## 4. Paste sessions keys in the genesis configuration file

An example of genesis configuration file: `resources/gdev.json`

## 5. Generate raw spec

```bash
./scripts/gen-live-network-raw-spec.sh CURRENCY "<path/to/your/genesis/config/file>"
```

## 6. Generate the docker compose and prepare nodes keys

```bash
./scripts/create-live-network.sh "<your secret phrase>" CURRENCY "<path/to/dist/folder>"
```

The distribution folder can then be copied to a server, then the compose must be launched from the
root of the distribution folder:

```bash
scp -r -P SSH_PORT "<path/to/dist/folder>" user@ip:/remote/dist/path
cd "<path/to/dist/folder>"
docker-compose up -d
```

Then, on the server:

```bash
cd "<path/to/dist/folder>"
docker-compose up -d
```
