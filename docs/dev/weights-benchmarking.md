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


## How to Write Benchmarks

### Calls

Ensure that any extrinsic call is benchmarked using the most computationally intensive path, i.e., the worst-case scenario.

### Hooks

Benchmark each hook to determine the weight consumed by it; hence, it is essential to benchmark all possible paths.

### Handlers and Internal Functions

When designing handlers and internal functions, it is advisable to avoid having them return weight for the following reasons:

1. **Simplified Benchmarking**: Writing benchmarks for hooks or calls where handlers and internal functions are utilized becomes more straightforward.
2. **Reduced Benchmarking Complexity**: By directly measuring execution and overhead in a single pass, the number of benchmarks is minimized.
3. **Enhanced Readability**: Understanding that weight accounting occurs at the outermost level improves the overall readability of the code.

One notable exception is the internal functions called in hooks like `on_idle` or `on_initialize` that can be easier to benchmark separately when the hook contains numerous branching.
