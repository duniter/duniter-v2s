# Replace core2 with no_std_io2

This copy of `multihash 0.17.0` replaces its `core2 0.4.0` dependency with
`no_std_io2 0.9.4`. RustSec identifies the latter as a maintained alternative in
[RUSTSEC-2026-0105](https://rustsec.org/advisories/RUSTSEC-2026-0105).
The root `[patch.crates-io]` applies it to litep2p's existing multihash dependency.
It does not replace the separate `multihash 0.19` dependency.

The base is the crates.io archive of multihash 0.17.0, whose recorded upstream
commit is [`dcd8e39409d15ba00104c86b22da82b919cbe0ef`](https://github.com/multiformats/rust-multihash/tree/dcd8e39409d15ba00104c86b22da82b919cbe0ef).
Source files, tests, examples, benchmarks, and the MIT license come from that
release. `Cargo.toml` starts from its published `Cargo.toml.orig`.

Changes:

- Rename the dependency and its `alloc` feature forwarding to `no_std_io2`.
- Rename the three conditional `no_std` imports in `error.rs`, `hasher_impl.rs`,
  and `multihash.rs`.
- Resolve `multihash-derive` from crates.io instead of its unpublished local path.
- Make this directory a standalone workspace for upstream tests.
- Enable `rand/small_rng` for the upstream test helper, which otherwise fails
  to compile under isolated Cargo feature resolution.

The `std` implementation already uses `std::io` directly. The patch does not
change hash computation, multihash encoding, or network protocol formats.
The `no_std` implementation uses the fork's I/O traits.

Run from the repository root after SDK preparation:

```sh
CARGO_TARGET_DIR=target/patch-tests ./scripts/cargo_with_vendor.sh test --manifest-path patches/multihash/Cargo.toml
CARGO_TARGET_DIR=target/patch-tests ./scripts/cargo_with_vendor.sh check --manifest-path patches/multihash/Cargo.toml --lib --no-default-features
CARGO_TARGET_DIR=target/patch-tests ./scripts/cargo_with_vendor.sh check --manifest-path patches/multihash/Cargo.toml --lib --no-default-features --features alloc
```

The standalone workspace has its own ignored lockfile for upstream development
dependencies. Production builds use the repository's root `Cargo.lock`.
Remove this patch when litep2p no longer requires multihash 0.17, or when its
supported multihash release removes core2 upstream.
