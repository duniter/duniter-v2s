# Xtask

We choose [`xtask`](https://github.com/matklad/cargo-xtask/) to run Rust scripts using `cargo`. To build these scripts, just run:

```bash
cargo xtask # this will build the scripts and show the available commands
```

These scripts mainly deal with runtime operations.

## Doc

```
Usage: xtask <COMMAND>

Commands:
  build                Build duniter binary
  gen-calls-doc        Generate calls documentation
  inject-runtime-code  Inject runtime code in raw specs
  release-runtime      Release a new runtime
  test                 Execute unit tests and integration tests End2tests are skipped
  help                 Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
