# How to build duniter-v2s for arm

1. Create a docker image that contains the build environment

```bash
docker build -t duniter-v2s-arm-builder -f docker/cross-arm.Dockerfile .
```

2. Use this docker image to cross-compile duniter-v2s for armv7

```bash
./scripts/cross-build-arm.sh
```

then, get the final binary at `target/armv7-unknown-linux-gnueabihf/release/duniter`.
