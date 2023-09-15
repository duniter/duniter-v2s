# Upgrade Substrate

We need to keep up to date with Substrate. Here is an empirical guide.

Let's say for the example that we want to upgrade from `v0.9.26` to `v0.9.32`.

## Upgrade Substrate fork
1. Checkout the latest Substrate Polkadot branch `polkadot-v0.9.xy` in [our Substrate](https://github.com/duniter/substrate/)
2. Create a new branch `duniter-substrate-v0.9.xy`
3. Cherry-pick `3ad5e1a1fdac5398d4911272a34e1f16c1bdd8f0`, `87ef489034ca2b15c4f30da387e06b1f6716d9a2` and `c36ab4f32454318a47777b24b6533c44121fc10b`
4. Run `cargo check` to check the cherry-picking

## Upgrade Subxt fork

1. Checkout the currently used branch in [our Subxt fork](https://github.com/duniter/subxt), e.g. `duniter-substrate-v0.9.26`
2. Create a new branch `duniter-substrate-v0.9.32`
3. Fetch the [upstream repository](https://github.com/paritytech/subxt)
4. Rebase on an upstream stable branch matching the wanted version

## Upgrade Duniter

1. Replace `duniter-substrate-v0.9.26` with `duniter-substrate-v0.9.32` in `Cargo.toml`
2. Update the `rust-toolchain` file according to [Polkadot release notes](https://github.com/paritytech/polkadot/releases)
	* Tip: To save storage space on your machine, do `rm target -r` after changing the rust toolchain version and before re-building the project with the new version.
3. While needed, iterate `cargo check`, `cargo update` and upgrading dependencies to match substrate's dependencies
4. Fix errors in Duniter code
	* You may need to check how Polkadot is doing by searching in [their repo](https://github.com/paritytech/polkadot). Luckily, the project structure and Substrate patterns are close enough to ours.
	* Some errors may happen due to two semver-incompatible versions of a same crate being used. To check this, use `cargo tree -i <crate>`. Update the dependency accordingly, then do `cargo update`.
5. As always, don't forget to `clippy` once you're done with the errors.
6. Test benchmarking:  
	`cargo run --features runtime-benchmarks -- benchmark overhead --chain=dev --execution=wasm --wasm-execution=interpreted-i-know-what-i-do --weight-path=. --warmup=10 --repeat=100`