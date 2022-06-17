# Duniter live tests

Kind of tests that run against a live chain!

## Sanity tests

Test suite that verifies the consistency of the onchain storage.

### Run sanity tests

1. Checkout the git tag of the runtime that you want to check
2. run the tests again the default network of the specified runtime type: `cargo sanity-RUNTIME_TYPE`

`RUNTIME_TYPE` should be replaced by `gdev`, `gtest` or `g1`.

#### Custom RPC endpoint

You can choose to use another RPC endpoint by setting the environment variable `WS_RPC_ENDPOINT`.
This is also the only way to test against a different network that the default one.

#### run against a specific block

You can choose to use run the sanity tests against a specific block by setting the environment
variable `AT_BLOCK_NUMBER`.

**Be careful: this would require to use an archive node.**

### Contribute to sanity tests

The code is in the file `live-tests/tests/sanity_RUNTIME_TYPE.rs`

There is 3 different parts:

1. Runtime types definitions
2. Collect storage data
3. Verify consistency of collected data
