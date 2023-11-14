# How to replay a block

You can use `try-runtime` subcommand to replay a block against a real state from a live network.

1. Checkout the git tag of the runtime version at the block you want to replay
2. Build duniter with feature `try-runtime`: `cargo build --features try-runtime`
3. Find the hash of the block to replay
4. Choose an RPC endpoint without path (try-runtime not support path)
5. Replay the block a first time to get the state:

```
duniter try-runtime --execution=Native execute-block --block-at 0x2633026e3e428b010cfe08d215b6253843a9fe54db28748ca56de37e6a83c644 live -s tmp/snapshot1 -u ws://localhost:9944
```

6. Then, replay the block as many times as you need against your local snapshot:

```
duniter try-runtime --execution=Native execute-block --block-at 0x2633026e3e428b010cfe08d215b6253843a9fe54db28748ca56de37e6a83c644 --block-ws-uri ws://localhost:9944 snap -s tmp/snapshot1
```

try-runtime does not allow (for now) to store the block locally, only the storage can be stored.
