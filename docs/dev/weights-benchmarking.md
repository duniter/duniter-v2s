# Weights benchmarking

## What is the reference machine?

For now (09/2022), it's a `Raspberry Pi 4 Model B - 4GB` with an SSD connected via USB3.

To cross-compile the benchmarks binary for armv7:

```
./scripts/cross-build-arm.sh --features runtime-benchmarks
```

The cross compiled binary is generated here: `target/armv7-unknown-linux-gnueabihf/release/duniter`

## How to benchmarks weights of a Call/Hook/Pallet

1. Create the benchmarking tests, see commit 31057e37be471e3f27d18c63818d57cc907b4b4f for a
complete real example.
2. Run the benchmark test on your local machine:
`cargo test -p <pallet> --features runtime-benchmarks`
3. If the benchmark tests compiles and pass, compile the binary with benchmarks on your local
machine: `cargo build --release --features runtime-benchmarks`
4. Run the benchmarks on your local machine (to test if it work with a real runtime). The command
is: `duniter benchmark pallet --chain=CURRENCY-dev --steps=50 --repeat=20 --pallet=pallet_universal_dividend --extrinsic=* --execution=wasm --wasm-execution=compiled --heap-pages=4096 --header=./file_header.txt --output=.`
5. If it worked, use the generated file content to create or update by hand the `WeightInfo` trait and the `()` dummy implementation. Then use the `WeightInfo` tarit in the real code of the pallet. See 79e0fd4bf3b0579279fc957da5e2fdfc6d8a17fa for a
complete real example.
6. Redo steps `3.` and `4.` on the reference machine.
7. Put the automatically generated file on `runtime/common/src/weights` and use it in the runtimes configuration.
See cee7c3b2763ba402e807f126534d9cd39a8bd025 for a complete real example.

Note 1: You *must* replace `CURRENCY` by the currency type, or for ĞDev use directly `--chain=gdev-benchmark`.

Note 2: If the reference machine does not support wasmtime, you should replace `--wasm-execution=compiled`
by `--wasm-execution=interpreted-i-know-what-i-do`.

## Generate base block benchmarking

1. Build binary for reference machine and copy it on reference machine.
2. Run base block benchmarks command:

```
./duniter benchmark overhead --chain=gdev --execution=wasm --wasm-execution=interpreted-i-know-what-i-do --weight-path=. --warmup=10 --repeat=100
```

3. Copy the generated file `block_weights.rs` in the codebase in folder `runtime/common/src/weights/`.
4. Commit changes and open an MR.

## Generate storage benchmarking

1. Build binary for reference machine and copy it on reference machine.
2. Copy a DB on reference machine (on ssd), example: `scp -r -P 37015 tmp/t1 pi@192.168.1.188:/mnt/ssd1/duniter-v2s/`
3. Run storage benchmarks command, example:

```
./duniter benchmark storage -d=/mnt/ssd1/duniter-v2s/t1 --chain=gdev --mul=2 --weight-path=. --state-version=1
```

4. Copy the generated file `paritydb_weights.rs` in the codebase in folder `runtime/common/src/weights/`.
5. Commit changes and open an MR.
