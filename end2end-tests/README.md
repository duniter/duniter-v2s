# Duniter end-to-end tests

Cucumber scenarios exercise an isolated gtest node through RPC. Each scenario
starts a temporary `gt_local` chain with manual sealing. `gt_local` aliases `gtest_local`. Transactions and empty blocks
advance chain time without waiting for real six-second intervals.

## Run the suite

Prepare the SDK as required by the repository, then run:

```sh
./scripts/cargo_with_vendor.sh cucumber-build
./scripts/cargo_with_vendor.sh cucumber
```

The node and the tests must both use `fast` and `constant-fees`. The build alias
selects gtest and includes the distance oracle. Set `DUNITER_BINARY_PATH` to use
another locally built node. Set `DUNITER_END2END_TESTS_SPAWN_NODE_TIMEOUT` to
change the RPC startup deadline in seconds. The default is 30 seconds.

Filter scenarios with `cargo cucumber -i 'monetary*'`. Logs are written to
`duniter-v2s-<rpc-port>.log` in this directory. Readiness is checked through RPC,
so the suite also works when `RUST_LOG` suppresses startup messages.

## Scenario preparation

The default preparation creates six members with five certifications each.
`@setup.wot` adds the membership states needed by the eligibility and
certification scenarios. `@setup.bad_distance` creates two cliques of six valid
members. Five certifiers from one clique cannot connect the newcomer to the
other six referees.

Preparations use the runtime's genesis builders to construct accounts,
identities, certification counters, membership schedules, and quotas together.
They install that state with `sudo(utility.batch_all(...))`. Consensus storage
and authoring keys remain those of the temporary local node. Reserved test
balances and the monetary mass include the treasury account and the funded sudo account.

Sudo uses `//Alice` with ed25519 on `gt_local`. Preparation first unlinks this
account from its genesis identity so it cannot use a scenario member's quota.
Its account, nonce, and fee holds are then preserved. Scenario actors use sr25519. Scenarios use ordinary signed calls for the
behavior under test. The runner checks transaction success and the inner
`Sudo::Sudid` result. A deliberately rejected transaction must assert its
expected error instead of suppressing errors for the whole scenario.

Genesis authorities have non-expiring memberships and received certifications.
To test smith expiration, add the smith after genesis during scenario preparation.

The setup never changes runtime parameters. The five-certification threshold
is the production threshold. Wait steps follow the UD schedule and distance
period transitions, with bounded block production. A distance cycle uses three
pools, each with the runtime's fast evaluation period.

## Distance coverage

The suite computes distance with the real oracle. It submits the computed result
through the privileged distance call, then waits for application by the runtime.
This covers the calculation and application, but not publication through the
node's distance inherent.

## Metadata

Regenerate production metadata with
`METADATA_CHAINS="g1 gtest" ./scripts/generate_all_metadata.sh`. Gtest metadata
supports the typed clients for the node, fixtures, and oracle. Run
`python3 scripts/check_runtime_compatibility.py` to check the permitted
differences between the production artifacts.
