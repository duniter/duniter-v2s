# Polkadot Upgrade Guide

ParityTech frequently releases upgrades of the polkadot-sdk. For each upgrade, Duniter should be upgraded following the instructions below. These instructions are based on upgrading from version 1.8.0 to 1.9.0.

## 1. Upgrade the duniter-polkadot-sdk

* Clone the repository: `git clone git@github.com:duniter/duniter-polkadot-sdk.git`
* Set the upstream repository: `git remote add upstream git@github.com:paritytech/polkadot-sdk.git`
* Fetch the latest released version: `git fetch --tag polkadot-v1.9.0`
* Create a new branch: `git checkout -b duniter-polkadot-v1.9.0`
* Rebase the branch, keeping only specific commits: "fix treasury benchmarks when no SpendOrigin", "allow manual seal to produce non-empty blocks with BABE", "add custom pallet-balance GenesisConfig", and "remove pallet-balances upgrade_account extrinsic", "remove all paritytech sdk dependencies".
* Push the new branch: `git push`

## 2. Upgrade duniter subxt

* Clone the repository: `git clone git@github.com:duniter/subxt.git`
* Set the upstream repository: `git remote add upstream git@github.com:paritytech/subxt.git`
* If a new version compatible with the polkadot-sdk version used in duniter is available, upgrade Subxt using the same method as duniter-polkadot-sdk.
* In the `Cargo.toml` file of duniter subxt, change the GitHub path pointing to the paritytech polkadot-sdk, or old duniter-polkadot-sdk to the new duniter-polkadot-sdk.
* Run `cargo upgrade` and push the branch.

## 3. Upgrade Dependencies

Ensure that the dependencies for [Arkwork](https://github.com/duniter/arkworks-substrate) and [ring-vrf bandersnatch-vrfs](https://github.com/duniter/ring-vrf/) only depend on the duniter-polkadot-sdk. When upgrading these repositories, make sure to cherry-pick the "use duniter polkadot-sdk" commit.

## 4. Upgrade repository

* In the `Cargo.toml` file of Duniter, change the version number from 1.8.0 to 1.9.0 for all polkadot-sdk dependencies. Also, change the version for Subxt. `find . -type f -name "Cargo.toml" -exec sed -i'' -e 's/polkadot-v1.8.0\/polkadot-v1.9.0/g' {} +`.
* Upgrade the version number of all crateio dependencies to ensure compatibility with those used in the polkadot-sdk, see the node template at: [Node Template](https://github.com/paritytech/polkadot-sdk/blob/master/templates/solochain/node/Cargo.toml) (choose the correct branch/tag).

At this point, two cases may arise:

1. If the upgrade only adds some types and minor changes, add the types in the pallet configuration, replace the offending `WeightInfo`, and delete the corresponding weights files until they can be regenerated.

2. If there are many breaking changes, it is recommended to break down the process:

    * Start by correcting errors on individual pallets using `cargo check -p my_pallet` to identify and rectify any errors. Then, test using `cargo test -p my_pallet` and benchmark using `cargo test -p my_pallet --feature runtime-benchmark`.
    * After correcting all pallets, fix the runtimes using the same approach: check for trait declarations added or removed in each pallet configuration, and use `cargo check -p runtime`, `cargo test -p runtime`, and `cargo test -p runtime --feature runtime-benchmark`.
    * Repeat this process with the node part, the distance-oracle, all the tests, xtask, and the client.
    * Conclude the process by executing all benchmarks using the command `scripts/run_all_benchmarks.sh`.

## 4. Troubleshooting

As Duniter may sometimes be the only chain implementing advanced features, such as manual sealing, not many references can be found. However, the following projects may be useful:

* Node template for general up-to-date implementation: [Node Template](https://github.com/paritytech/polkadot-sdk/tree/master/templates)
* Acala: [Acala](https://github.com/AcalaNetwork/Acala), which also uses manual sealing add a similar node implementation.
