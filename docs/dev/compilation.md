# Compilation

Duniter is compiled using the Rust compiler. For a general overview, refer to the [Rustc Dev Guide](https://rustc-dev-guide.rust-lang.org/overview.html).

Substrate and Duniter provide a set of features enabling or disabling parts of the code using conditional compilation. More information on conditional compilation can be found [here](https://doc.rust-lang.org/reference/conditional-compilation.html), or by enabling or disabling compilation of packages. Below is a list of all available features:

## External

- **runtime-benchmarks**: Compiles the runtime with benchmarks for extrinsics benchmarking.
- **try-runtime**: Compiles the runtime for tests and verifies operations in a simulated environment.
- **std**: Enables the Rust standard library.

## Duniter

- **gtest**: Sets `gtest-runtime` and `std` used to build the test chain.
- **g1**: Sets `g1-runtime` and `std` used to build the production chain.
- **constant-fees**: Uses a constant and predictable weight-to-fee conversion only for testing.
- **embed**: Enables hardcoded live chainspecs loaded from "../specs/gtest-raw.json" file.
- **native**: Compiles the runtime into native-platform executable only for debugging purposes.

Note: By default, Duniter will be compiled using the `gtest` feature and including the compilation of the distance oracle. The `gtest` and `g1` node features are mutually exclusive. Disable default features when selecting g1:

- `cargo build --no-default-features --features gtest`
- `cargo build --no-default-features --features g1`
- `cargo build --no-default-features -p distance-oracle --features gtest,std,standalone`

The two runtimes share their implementation, parameters, and weights. See
[the runtime guide](../../runtime/README.md). Compile local test nodes with
`cargo cucumber-build` and run the end-to-end suite with `cargo cucumber`.
