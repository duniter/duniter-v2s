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
  gen-doc              Generate documentation (calls and events)
  inject-runtime-code  Inject runtime code in raw specs
  release-runtime      Release a new runtime
  update-raw-specs     Update raw specs locally with the files published on a Release
  create-asset-link    Create asset in a release
  test                 Execute unit tests and integration tests End2tests are skipped
  g1-data              Generate G1 data using Docker and py-g1-migrator
  help                 Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## G1 Data Generation

La commande `g1-data` permet de générer le genesis.json de la G1 en utilisant Docker et py-g1-migrator, équivalent à la tâche CI `g1_data`.

### Utilisation

```bash
cargo xtask g1-data --dump-url <URL_DUMP_G1>
```

### Exemple

```bash
cargo xtask g1-data --dump-url "https://dl.cgeek.fr/public/auto-backup-g1-duniter-1.8.7_2025-10-06_18-00.tgz"
```

Cette commande :
1. Télécharge le dump G1 depuis l'URL fournie
2. Lance un conteneur Docker avec py-g1-migrator
3. Extrait et convertit les données G1 vers les formats Substrate et Squid
4. Génère les fichiers `genesis.json`, `block_hist.json`, `cert_hist.json`, et `tx_hist.json`
5. Copie les fichiers dans le répertoire `output/`
