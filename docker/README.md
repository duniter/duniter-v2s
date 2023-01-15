# duniter/duniter-v2s

Duniter is the software that supports the [Ğ1 libre-currency blockchain](https://duniter.org/).

[Duniter v2s](https://git.duniter.org/nodes/rust/duniter-v2s) is a complete rewrite of Duniter based on the Substrate / Polkadot framework. **This is alpha state work in progress.**

# Minimal docker-compose file for an RPC (non validator) node

```
version: "3.5"

services:
  duniter-rpc:
    image: duniter/duniter-v2s:latest
    restart: unless-stopped
    ports:
      # Prometheus endpoint
      - 9615:9615
      # rpc via http
      - 9933:9933
      # rpc via websocket
      - 9944:9944
      # p2p
      - 30333:30333
    volumes:
      - data-rpc:/var/lib/duniter/
    environment:
      - DUNITER_CHAIN_NAME=gdev
      - DUNITER_NODE_NAME=<my-node-name>

volumes:
  data-rpc:
```

# Minimal docker-compose file for a validator node

```
version: "3.5"

services:
  duniter-validator:
    image: duniter/duniter-v2s:latest
    restart: unless-stopped
    ports:
      # Prometheus endpoint
      - 9615:9615
      # p2p
      - 30333:30333
    volumes:
      - data-validator:/var/lib/duniter/
    environment:
      - DUNITER_CHAIN_NAME=gdev
      - DUNITER_VALIDATOR=true
      - DUNITER_NODE_NAME=<my-validator-node-name>

volumes:
  data-validator:
```

# Environment variables

| Name | Description | Default |
| ---- | ----------- | ------- |
| `DUNITER_NODE_NAME` | The node name. This name will appear on the Substrate telemetry server when telemetry is enabled. | Random name |
| `DUNITER_CHAIN_NAME` | The currency to process. "gdev" uses the embeded chainspec. A path allows to use a local json raw chainspec. | `dev` (development mode) |
| `DUNITER_PUBLIC_ADDR` | The libp2p public address base. See [libp2p documentation](https://docs.libp2p.io/concepts/fundamentals/addressing/). This variable is useful when the node is behind a reverse-proxy with its ports not directly exposed.<br>Note: the `p2p/<peer_id>` part of the address shouldn't be set in this variable. It is automatically added by Duniter. | duniter-v2s guesses one from the node's IPv4 address. |
| `DUNITER_LISTEN_ADDR` | The libp2p listen address. See [libp2p documentation](https://docs.libp2p.io/concepts/fundamentals/addressing/). This variable is useful when running a validator node behind a reverse proxy, to force the P2P end point in websocket mode with:<br>  `DUNITER_LISTEN_ADDR=/ip4/0.0.0.0/tcp/30333/ws` | Non validator node: `/ip4/0.0.0.0/tcp/30333/ws`<br>Validator node: `/ip4/0.0.0.0/tcp/30333
| `DUNITER_RPC_CORS` | Value of the polkadot `--rpc-cors` option. | `all` |
| `DUNITER_VALIDATOR` | Boolean (`true` / `false`) to run the node in validator mode. Configure the polkadot options `--validator --rpc-methods Unsafe`. | `false` |
| `DUNITER_DISABLE_PROMETHEUS` | Boolean to disable the Prometheus endpoint on port 9615. | `false` |
| `DUNITER_DISABLE_TELEMETRY` | Boolean to disable connecting to the Substrate tememetry server. | `false` |

# Other duniter options

You can pass any other option to duniter using the `command` docker-compose element:
```
    command:
      # workaround for substrate issue #12073
      # https://github.com/paritytech/substrate/issues/12073
      - "--wasm-execution=interpreted-i-know-what-i-do"
```
