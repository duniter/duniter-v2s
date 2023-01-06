# duniter/duniter-v2s

Duniter is the software that supports the [Ğ1 libre-currency blockchain](https://duniter.org/).

[Duniter v2s](https://git.duniter.org/nodes/rust/duniter-v2s) is a compete rewrite of Duniter based on the Substrate / Polkadot framework. **This is alpha state work in progress.**

# Minimal docker-compose file for an RPC (non validator) node

```
version: "3.5"

services:
  duniter-rpc:
    image: duniter/duniter-v2s:latest
    restart: unless-stopped
    ports:
      # telemetry
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
      # telemetry
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

* `DUNITER_NODE_NAME`
  The node name. Default: random name
* `DUNITER_CHAIN_NAME`
  The currency to process. Should be "gdev" at the current duniter-v2s development stage.
* `DUNITER_PUBLIC_ADDR`
  The libp2p public address base. See [libp2p documentation](https://docs.libp2p.io/concepts/fundamentals/addressing/). When this variable is not defined duniter-v2s guesses one from the node's IPv4 address.
  This variable is useful when the node is behind a reverse-proxy with its ports not directly exposed.
  Note: the `p2p/<peer_id>` part of the address shouldn't be set in this variable. It is automatically added by Duniter.
* `DUNITER_LISTEN_ADDR`
  The libp2p listen address. See [libp2p documentation](https://docs.libp2p.io/concepts/fundamentals/addressing/). This variable is useful when running a validator node behind a reverse proxy, to force the P2P end point in websocket mode with:
  `DUNITER_LISTEN_ADDR=/ip4/0.0.0.0/tcp/30333/ws`
* `DUNITER_RPC_CORS`
  Value of the polkadot `--rpc-cors` option. Defaults to `all`.
* `DUNITER_VALIDATOR`
  Boolean (`true` / `false`) to run the node in validator mode. Defaults to `false`.
  Configure the polkadot options `--validator --rpc-methods Unsafe`.
* `DUNITER_DISABLE_PROMETHEUS`
  Boolean to disable the telemetry entry point. Defaults to false.
