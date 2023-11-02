# Weights benchmarking

## What is the reference machine?

For now (09/2022), it's a `Raspberry Pi 4 Model B - 4GB` with an SSD connected via USB3.

To cross-compile the benchmarks binary for armv7:

```
./scripts/cross-build-arm.sh --features runtime-benchmarks
```

The cross compiled binary is generated here: `target/armv7-unknown-linux-gnueabihf/release/duniter`

## How to benchmarks weights of a Call/Hook/Pallet

1. Create the benchmarking tests. See commit f5f2ae969ac592ba9957b0e40e18d6e4b0048473 for a
complete real example.

2. Run the benchmark test on your local machine:

```
cargo test -p <pallet> --features runtime-benchmarks
```

3. If the benchmark tests compiles and pass, compile the binary with benchmarks on your local
machine:

```
cargo build --release --features runtime-benchmarks
```

4. Run the benchmarks on your local machine (to test if it work with a real runtime). See 0d1232cd0d8b5809e1586b48376f8952cebc0d27 for a complete real example. The command is:

```
duniter benchmark pallet --chain=CHAINSPEC --steps=50 --repeat=20 --pallet=<pallet> --extrinsic=* --execution=wasm --wasm-execution=compiled --heap-pages=4096 --header=./file_header.txt --output=./runtime/common/src/weights/
```

5. Use the generated file content to create the `WeightInfo` trait and the `()` dummy implementation in `pallets/<pallet>/src/weights.rs`. Then use the `WeightInfo` trait in the real code of the pallet. See 62dcc17f2c0b922e883fbc6337a9e7da97fc3218 for a complete real example.

6. Redo steps `3.` and `4.` on the reference machine.

7. Use the `runtime/common/src/weights/pallet_<pallet>.rs` generated on the reference machine in the runtimes configuration. See  af62a3b9cfc42d6653b3a957836f58540c18e65a for a complete real example.

Note 1: Use relevant chainspec for the benchmarks in place of `CHAINSPEC`, for example `--chain=dev`.

Note 2: If the reference machine does not support wasmtime, you should replace `--wasm-execution=compiled`
by `--wasm-execution=interpreted-i-know-what-i-do`.

## Generate base block benchmarking

1. Build binary for reference machine and copy it on reference machine.
2. Run base block benchmarks command:

```
duniter benchmark overhead --chain=dev --execution=wasm --wasm-execution=compiled --weight-path=./runtime/common/src/weights/ --warmup=10 --repeat=100
```

3. Commit changes and open an MR.

## Generate storage benchmarking

1. Build binary for reference machine and copy it on reference machine.
2. Copy a DB on reference machine (on ssd), example: `scp -r -P 37015 tmp/t1 pi@192.168.1.188:/mnt/ssd1/duniter-v2s/`
3. Run storage benchmarks command, example:

```
duniter benchmark storage -d=/mnt/ssd1/duniter-v2s/t1 --chain=gdev --mul=2 --weight-path=. --state-version=1
```

4. Copy the generated file `paritydb_weights.rs` in the codebase in folder `runtime/common/src/weights/`.
5. Commit changes and open an MR.
