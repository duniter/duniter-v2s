# Ğ1 runtime

This production entry point instantiates `../shared/runtime.rs` with
`spec_name = "g1"` and without sudo. Pallet index 20 remains vacant.
The live sudo key is already `None`, so no key-removal migration is needed.

See [the runtime guide](../README.md) for shared parameters, weights, and validation.
