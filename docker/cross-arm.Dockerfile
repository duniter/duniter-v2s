FROM rust:latest

RUN dpkg --add-architecture armhf && \
  apt-get update && apt-get upgrade -y && \
  apt-get install -y aptitude && \
  aptitude install -y \
  gcc-arm-linux-gnueabihf \
  g++-arm-linux-gnueabihf \
  pkg-config \
  cmake \
  libssl-dev \
  git \
  clang \
  libclang-dev \
  libssl-dev:armhf

# Install nightly with armv7 and  w32-u-u
RUN rustup install nightly-2021-11-12-x86_64-unknown-linux-gnu && \
  rustup target add armv7-unknown-linux-gnueabihf --toolchain \
  nightly-2021-11-12-x86_64-unknown-linux-gnu && \
  rustup target add wasm32-unknown-unknown --toolchain \
  nightly-2021-11-12-x86_64-unknown-linux-gnu && \
  cargo +nightly-2021-11-12-x86_64-unknown-linux-gnu install --git \
  https://github.com/alexcrichton/wasm-gc --force

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER arm-linux-gnueabihf-gcc
ENV PKG_CONFIG_ALLOW_CROSS 1
ENV PKG_CONFIG_PATH /usr/lib/arm-linux-gnueabihf/pkgconfig/


# Disallow the `pkg-config` crate to look for the config for zlib, because build.rs of `libz-sys`
# gets confused and pulls the system-wide library (i.e. of the host) instead of the target when
# cross-compiling. This essentially leads to static linking of zlib.
#
# Alternatively, we can supply LIBZ_SYS_STATIC=1. Weirdly enough, installing libgtk-3-dev:armhf
# also solves the problem somehow.
#
# Here is the related issue: https://github.com/rust-lang/libz-sys/issues/49
ENV ZLIB_NO_PKG_CONFIG 1

# This is for compiling GUI apps.
# RUN aptitude install -y libasound2-dev:armhf libgtk-3-dev:armhf libsdl2-dev:armhf

RUN useradd rust --user-group --create-home --shell /bin/bash --groups sudo
WORKDIR /home/rust/src
