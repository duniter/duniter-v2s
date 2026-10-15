# Gtest runtime

This entry point instantiates `../shared/runtime.rs` with `spec_name = "gtest"`
and sudo at pallet index 20. All business parameters and weights match G1.

All runtime and end-to-end tests run on gtest. Local test builds enable `fast`
and `constant-fees`. Public network builds keep both features disabled.

See [the runtime guide](../README.md) for artifact compatibility checks.
