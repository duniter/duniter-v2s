# How to Build the Duniter RPM Package

1. Install dependencies:
   ```sh
   # Fedora
   sudo dnf install clang cmake protobuf-compiler openssl-devel
   ```
2. Compile the project:
   ```sh
   cargo build --release
   ```
3. Install `cargo-generate-rpm`:
   ```sh
   cargo install cargo-generate-rpm
   ```
4. Build the package:
   ```sh
   cargo generate-rpm -p node
   ```
5. The `.rpm` package will be located in the `target/generate-rpm` folder.
