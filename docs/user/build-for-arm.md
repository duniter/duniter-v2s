# How to build duniter-v2s for arm

Cross-compile Duniter to arm (e.g. Raspberry Pi).

## With Docker

1. Create a docker image that contains the build environment

```bash
docker build -t duniter-v2s-arm-builder -f docker/cross-arm.Dockerfile .
```

2. Use this docker image to cross-compile duniter-v2s for armv7

```bash
./scripts/cross-build-arm.sh
```

then, get the final binary at `target/armv7-unknown-linux-gnueabihf/release/duniter`.

## Without Docker

**Warning**: armv7 (default for Raspberry Pi) is **not** supported. Linux on RPi can be easily switched to aarch64, please search how to do so on the Internet.

This produces a musl build: the resulting executable is static, hence more portable than a dynamic one. It will be compatible with systems older than the compilation host.

```bash
# Install the tools
rustup target add aarch64-unknown-linux-musl --toolchain nightly-2023-08-23-x86_64-unknown-linux-musl
sudo dpkg --add-architecture arm64
sudo apt update
sudo apt install musl-dev:arm64 musl-tools g++-aarch64-linux-gnu gcc-aarch64-linux-gnu

# Cross-compile
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc cargo build --target=aarch64-unknown-linux-musl --release
```
