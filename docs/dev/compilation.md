# Compilation

Duniter is compiled using the Rust compiler. For a general overview, refer to the [Rustc Dev Guide](https://rustc-dev-guide.rust-lang.org/overview.html).

Substrate and Duniter provide a set of features enabling or disabling parts of the code using conditional compilation. More information on conditional compilation can be found [here](https://doc.rust-lang.org/reference/conditional-compilation.html), or by enabling or disabling compilation of packages. Below is a list of all available features:

## External

- **runtime-benchmarks**: Compiles the runtime with benchmarks for extrinsics benchmarking.
- **try-runtime**: Compiles the runtime for tests and verifies operations in a simulated environment.
- **std**: Enables the Rust standard library.

## Duniter

- **gdev**: Sets `gdev-runtime` and `std` used to build the development chain.
- **gtest**: Sets `gtest-runtime` and `std` used to build the test chain.
- **g1**: Sets `g1-runtime` and `std` used to build the production chain.
- **constant-fees**: Uses a constant and predictable weight-to-fee conversion only for testing.
- **embed**: Enables hardcoded live chainspecs loaded from "../specs/gtest-raw.json" file.
- **native**: Compiles the runtime into native-platform executable only for debugging purposes.

Note: By default, Duniter will be compiled using the `gdev` feature and including the compilation of the distance oracle. Since the three Duniter chains are mutually exclusive, it is mandatory to disable the default feature to compile `gtest` and `g1` as follows:

- `cargo build --no-default-features --features gtest`
- `cargo build --no-default-features --features g1`
- `cargo build --no-default-features -p distance-oracle --features std`
