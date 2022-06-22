#!/bin/bash

docker run --rm -it \
	--env BINDGEN_EXTRA_CLANG_ARGS="--sysroot=/usr/arm-linux-gnueabihf -D__ARM_PCS_VFP -mfpu=vfp -mfloat-abi=hard" \
	--env CARGO_HOME=/home/rust/src/arm-build \
	-v "$(pwd)":/home/rust/src duniter-v2s-arm-builder \
	cargo build --target=armv7-unknown-linux-gnueabihf --release "$@"
