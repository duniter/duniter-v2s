# Runtimes

The `gtest-runtime` and `g1-runtime` packages instantiate the same implementation
in `shared/runtime.rs`. Parameters and weights are shared too.

| Property | Gtest | G1 |
| --- | --- | --- |
| `spec_name` | `gtest` | `g1` |
| `impl_name` | `duniter` | `duniter` |
| Metadata-hash token symbol | `Ğ1` | `Ğ1` |
| Sudo pallet | Index 20 | Absent, index 20 remains vacant |

Sudo remains useful for automated scenario setup and local development, so Gtest
keeps it. G1 has no sudo pallet. Its live sudo key is already `None`, so this
change requires no migration to remove a key.
The two entry-point crates contain no separate business rules.
Their build scripts include `shared/build.rs`.

## Tests and production builds

Run runtime and end-to-end tests with gtest. The `fast` feature selects shorter
periods from the shared parameters. It keeps the five-certification membership
threshold and the six-second timestamp increment. `constant-fees` makes fee
assertions deterministic.

Build each network artifact in a separate Cargo invocation. Compile-time checks
reject mismatched test features in the shared dependency.

Production builds use neither feature. Generate both production metadata files
with `METADATA_CHAINS="g1 gtest" ./scripts/generate_all_metadata.sh`, then run
`python3 scripts/check_runtime_compatibility.py`. This compares constants, pallet
indexes, call and event encodings, storage types, runtime APIs, and transaction
extensions. It permits the different `spec_name` and gtest's sudo pallet.

A metadata comparison does not establish that test timings exercise production
periods. Changes to the shared duration definitions still require review.

## Weights

Both runtimes use `shared/weights/`. Generate common weights once using the gtest
benchmark runtime. The sudo weight implementation is included only by gtest.

Before upgrading an existing G1 chain, inspect scheduled calls and stored
preimages for calls to sudo. Removing the pallet makes those calls undecodable.
Replace any required operation with a governance call before the upgrade.
Runtime release validation uses a local fork as described in
[the upgrade procedure](../docs/dev/runtime-upgrade.md).

## Local chains

With a gtest node, `--chain gt_local` is an alias for `--chain gtest_local`.
Both produce the same spec, with ID and protocol ID `gtest_local`, one Alice
authority, five smiths, six identities, and sudo for `//Alice` in ed25519 by default.
`DUNITER_SUDO_ACCOUNT_CRYPTO=sr25519` selects the alternate local sudo key.

The genesis authorities have non-expiring memberships and received certifications.
A test of smith expiration must add its smith after genesis during scenario setup.

`gtest_local` is used by the [indexer sink e2e test](../node/tests/indexer_sink_e2e.rs).
`gtest_local2` is used by the [two-validator consensus helper](../scripts/run-gtest-local2.sh).

## G1 before and after the refactor

Prepare the SDK with `sdk_resolve`, then run:

```sh
./scripts/check_g1_refactor.sh <pre-refactor-commit>
```

Without an argument, the script uses `CI_MERGE_REQUEST_DIFF_BASE_SHA` in CI or
`git merge-base HEAD origin/master` locally. MR 416 runs this check in CI.
The script builds both G1 Wasm artifacts with `metadata-hash`, without `fast`,
`constant-fees`, or `runtime-benchmarks`. It saves the source revisions, local
patch, Wasm artifacts, metadata, and differential result in
`target/g1-refactor-comparison/`.

The metadata comparison permits only removal of sudo at index 20 and the
`impl_name` change from `duniter-g1` to `duniter`. All other version fields,
constants, pallet indexes, call and event encodings, storage types, runtime APIs,
and transaction extensions must match.

The differential test builds one fixture with the old Wasm genesis builder and
uses identical copies for both Wasm executors. It compares dispatch results,
finalized headers, complete storage roots, and raw storage entries, including
events. Both executors build and import the same blocks across a production
BABE epoch boundary. The fixture covers transfers, a failed transfer, an eventful
remark, UD creation and claiming, UD reevaluation, certification expiry, and
membership expiry. Upcoming schedules are seeded in genesis state. The compiled
production periods remain unchanged.

Separate signature probes verify that each runtime accepts its metadata digest
with the expected symbol, `Ğ` before and `Ğ1` after, and rejects both the wrong
symbol and the other runtime's digest. Ordinary differential transactions keep
metadata checking disabled and use identical encoded bytes.

This is regression coverage for these scenarios, not a proof for every possible
state and call. It does not replace the governance upgrade test on a local fork
of the live G1 network required for a runtime release.

### Metadata hash in G1 applications

Source audit on 2026-10-15, using the fetched default branches and their locked
dependencies. These findings concern the applications' current signing paths,
not every external signer or every installed release.

| Application | Signing path | Metadata hash |
| --- | --- | --- |
| [Cesium2 `bf97c218`](https://git.duniter.org/clients/cesium-grp/cesium2s/-/blob/bf97c2186f399b9e67f54425d3871f14ce465ac3/src/app/shared/substrate/extrinsic.utils.ts#L133) | `signAsync`, no hash or mode option, Polkadot.js 16.5.6 | Mode 0, hash absent. Also checked by constructing its default payload locally. |
| [Gecko `5149e583`](https://git.duniter.org/clients/gecko/-/blob/5149e583f5367c687444fcdf0170047fb45501a2/pubspec.lock) | durt2 1.10.1, [its `ExtrinsicBuilder` call](https://git.duniter.org/libs/durt/-/blob/8b9e86fd09d39c66d3c42f78217525518e389557/lib/src/services/duniter_service.dart#L558), Polkadart 1.1.0 | No hash override. `enableMetadataHash` defaults to false. |
| [Ginkgo `00317ed0`](https://git.duniter.org/vjrj/ginkgo/-/blob/00317ed0535bb54efbed859a8a54f48e0d905ac3/lib/g1/sign_and_send.dart#L176) | `SigningPayload` and `ExtrinsicPayload`, Polkadart 0.7.1 | Hash argument omitted. `metadataHash` defaults to `00`, which encodes mode 0 and no hash. |
| [g1cli `59de2875`](https://git.duniter.org/clients/rust/g1cli/-/blob/59de28757903973d6719d67dbbd8e8bd81003808/src/utils.rs#L79) | Default extrinsic parameters, Subxt v0.44.0 | [Subxt explicitly encodes mode 0 and `None`](https://github.com/paritytech/subxt/blob/v0.44.0/core/src/config/transaction_extensions.rs#L102). |

Changing the metadata-hash symbol therefore does not affect these signing paths.
A signer that enables metadata checking must use the new runtime metadata and
`Ğ1` symbol. Its previous digest cannot remain valid after sudo removal either.

The live G1 sudo key was also read at finalized block 3,186,257, hash
`0xf5dc584cf973c2ff389cbc6dcbac8c4f78613af68c0e756b13db8e5ce9c8f2a7`,
through `wss://rpc.duniter.org/`. `state_getStorage` returned `null` for
`twox128("Sudo") ++ twox128("Key")`, which means `Sudo::Key` is `None`.
That observation concerns live runtime 1100. The before/after test compares
version 1200 on the MR base with version 1200 on the source branch.
