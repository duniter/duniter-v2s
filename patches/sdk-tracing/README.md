# Allow the patched tracing subscriber

These two crates let Duniter use `tracing-subscriber 0.3.20`, the first release
that fixes [RUSTSEC-2025-0055](https://github.com/advisories/GHSA-xwfj-jgwm-7wp5).
The root `Cargo.toml` patches their SDK source.

Upstream source is copied from
[`duniter-polkadot-sdk` commit `e2f5d44770e741d708b5ee551cf152afccee0b54`](https://github.com/duniter/duniter-polkadot-sdk/tree/e2f5d44770e741d708b5ee551cf152afccee0b54):

| Local crate | SDK directory | Version |
| --- | --- | --- |
| `sc-tracing` | `substrate/client/tracing` | 45.0.0 |
| `sp-tracing` | `substrate/primitives/tracing` | 19.0.0 |

All Rust source, upstream tests, and benchmarks are unchanged. Each local
manifest expands SDK workspace inheritance into explicit values. SDK dependencies
still use `duniter-substrate-v1.22.3`, pinned by the root lockfile. The only
dependency requirement change is `tracing-subscriber =0.3.19` to `=0.3.20`.
The upstream license texts and source notices are retained here.
The nested `sc-tracing-proc-macro` crate is not copied; it remains a Git dependency.

A direct `[patch.crates-io]` pointing to subscriber 0.3.20 cannot satisfy the SDK's
exact 0.3.19 requirement. Patching these two consumers permits the actual released
0.3.20 crate without changing its version or carrying a subscriber backport.
The SDK's other subscriber requirements are transitive development dependencies,
which do not participate in Duniter's build.

`node/tests/log_sanitization.rs` exercises the SDK logger through `sc-cli` and
the `log` bridge. It checks that ESC, BEL, and a C1 control character cannot reach
terminal output unchanged. Run it from the repository root:

```sh
./scripts/cargo_with_vendor.sh test -p duniter --test log_sanitization
```

When updating the SDK, compare these copies against the new SDK source. Remove
both root patches and these directories once the SDK accepts subscriber 0.3.20
or newer. Do not retain these copies across an SDK update without that review.

The [review attached to the dependency-audit MR](https://git.duniter.org/-/project/520/uploads/9f036033d5ec0fef59e343b89a584ba5/dependency-vulnerabilities.md)
documents applicability, audit exceptions, and validation.
