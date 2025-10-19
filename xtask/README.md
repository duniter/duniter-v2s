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
  build                   Build duniter binary
  gen-doc                 Generate documentation (calls and events)
  inject-runtime-code     Inject runtime code in raw specs
  print-spec              Print the chainSpec published on given Network Release
  create-asset-link       Create asset in a release
  test                    Execute unit tests and integration tests End2tests are skipped
  network-g1-data         Generate G1 data using Docker and py-g1-migrator
  network-build-specs     Build network specs (reprend la tâche build_specs de la CI)
  network-build-runtime   Build network runtime (reprend la tâche build_network_runtime de la CI)
  network-create-release  Create network release (reprend la tâche create_network_release de la CI)
  client-build-raw-specs  Build raw specs (reprend la tâche build_raw_specs de la CI)
  client-docker-deploy    Docker deploy (reprend la tâche docker_deploy de la CI)
  client-create-release   Create client release (reprend la tâche create_client_release de la CI)
  client-build-rpm        Build RPM (reprend la tâche build_rpm de la CI)
  client-build-deb        Build DEB (reprend la tâche build_deb de la CI)
  runtime-build           Build runtime (reprend la tâche build_runtime de la CI)
  runtime-create-release  Create runtime release (reprend la tâche create_runtime_release de la CI)
  help                    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## 