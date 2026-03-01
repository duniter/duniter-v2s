# Runtime upgrade

Complete guide for applying a runtime upgrade on an existing Duniter network
(GTest, G1). This process does not restart the network: the new runtime is
applied live through a Technical Committee vote.

## When to perform a runtime upgrade?

A runtime upgrade is needed when pallet code or runtime configuration has
changed: new features, bug fixes, parameter changes, new benchmarks, etc.

Do not confuse with:

- **Network release**: bootstrapping a new network from scratch
  (see [release.md](./release.md#network-release))
- **Client release**: updating the node binary
  (see [release.md](./release.md#client-release))

A runtime upgrade does **not** require restarting nodes. The new WASM is stored
on-chain and nodes use it automatically.

## Prerequisites

- Docker (for srtool)
- Linux x86_64 machine recommended (srtool is amd64 only, slow under emulation
  on ARM)
- `GITLAB_TOKEN` if you want to publish the GitLab release
- Being a Technical Committee member to submit the proposal

## Step 1: Prepare the branch

Create a dedicated branch from `master`:

```bash
git checkout master && git pull
git checkout -b runtime/<network>-<new-version>
```

Example:

```bash
git checkout -b runtime/gtest-1100
```

## Step 2: Bump the spec_version

In `runtime/<network>/src/lib.rs`, increment `spec_version`:

```rust
spec_version: 1100,  // was 1000
```

The `spec_version` must be strictly greater than the version currently on-chain.
Nodes will reject a runtime with an identical or lower `spec_version`.

To check the on-chain version:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"state_getRuntimeVersion","params":[]}' \
  | websocat -n1 wss://<endpoint>
```

## Step 3: Storage migration (if needed)

If storage types have changed (new field, modified type, renamed or removed
storage), you need to write a migration script. Otherwise, skip this step.

Check by reviewing the pallet diff:

```bash
git diff master -- pallets/
```

Look for changes in `#[pallet::storage]` blocks. If no storage type has
changed, **no migration is needed**.

## Step 4: Commit and push

```bash
git add runtime/<network>/src/lib.rs
git commit -m "chore(<network>): bump spec_version to <new-version>"
git push -u origin runtime/<network>-<new-version>
```

## Step 5: Build the runtime WASM

```bash
rm -rf release/*
cargo xtask release runtime build <network>
```

The WASM file is generated at
`release/<network>_runtime.compact.compressed.wasm`.

The build uses srtool (Docker `paritytech/srtool`) to guarantee
reproducibility. The srtool version matches the Rust channel in
`rust-toolchain.toml`.

## Step 6: Publish the GitLab release (optional)

Create a milestone `runtime-<new-version>` on GitLab, then:

```bash
export GITLAB_TOKEN=...
cargo xtask release runtime create <network> runtime/<network>-<new-version>
```

See [release.md](./release.md#runtime-release) for details.

## Step 7: Submit on-chain via Technical Committee

The runtime upgrade is applied through `pallet_upgrade_origin` which requires a
**2/3 supermajority** of the Technical Committee.

### 7a. Create the preimage

On [Duniter Portal](https://duniter-portal.axiom-team.fr/) connected to the network:

1. **Governance > Preimages > Add preimage**
2. Select `upgradeOrigin` > `dispatchAsRootUncheckedWeight`
3. Inside, select `system` > `setCode`
4. Upload the WASM file
   (`release/<network>_runtime.compact.compressed.wasm`)
5. Copy the **preimage hash** and submit the transaction

### 7b. Submit the proposal to the Technical Committee

1. **Governance > Tech. committee > Proposals > Submit proposal**
2. Select `upgradeOrigin` > `dispatchAsRootUncheckedWeight` >
   `system` > `setCode`
3. Use the preimage hash from step 7a
4. Set the `lengthBound` (WASM file size in bytes)
5. Submit

### 7c. TC members vote

Each Technical Committee member should:

1. **Verify the blake2_256 hash** of the runtime before voting
   (see [verify-runtime-code.md](./verify-runtime-code.md))
2. Vote **Aye** on the proposal
3. Once the 2/3 threshold is reached, **close the motion**

The runtime upgrade takes effect at the **next block** after the motion is
executed.

## Step 8: Post-upgrade verification

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"state_getRuntimeVersion","params":[]}' \
  | websocat -n1 wss://<endpoint>
```

Verify that `specVersion` returns the new value.

Also check that the network keeps producing and finalizing blocks:

```bash
# Latest block
echo '{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}' \
  | websocat -n1 wss://<endpoint>

# Latest finalized block
echo '{"jsonrpc":"2.0","id":1,"method":"chain_getFinalizedHead","params":[]}' \
  | websocat -n1 wss://<endpoint>
```

## See also

- [release.md](./release.md) — Build and publish artifacts on GitLab
- [verify-runtime-code.md](./verify-runtime-code.md) — blake2_256 hash
  verification for TC voters
- [g1-production-launch.md](./g1-production-launch.md) — Launching a new
  network (separate process)
