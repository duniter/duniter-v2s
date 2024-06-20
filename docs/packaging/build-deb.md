# How to Build Duniter-V2S Debian Package

Compile packages for native integration for Debian-based systems.

## With Docker (on any system)

1. Install Docker and Docker Buildx.
2. Use the `scripts/build-deb.sh` script.
3. The `.deb` packages will be located in the `target/debian` folder.

## Without Docker (on a Debian-based system)

1. Install the necessary dependencies:
   ```sh
   sudo apt-get install -y clang cmake protobuf-compiler libssl-dev
   ```
2. Compile the project:
   ```sh
   cargo build --release
   ```
3. Install `cargo-deb`:
   ```sh
   cargo install cargo-deb
   ```
4. Build the Duniter node `.deb` package:
   ```sh
   cargo deb --no-build -p duniter
   ```
5. The `.deb` package will be located in the `target/debian` folder.
