# Workaround for https://github.com/containers/buildah/issues/4742
FROM --platform=$BUILDPLATFORM docker.io/library/debian:bullseye-slim AS target

# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------

# When building for a foreign arch, use cross-compilation
# https://www.docker.com/blog/faster-multi-platform-builds-dockerfile-cross-compilation-guide/
FROM --platform=$BUILDPLATFORM docker.io/library/rust:1-bullseye AS build
ARG BUILDPLATFORM
ARG TARGETPLATFORM

# Debug
RUN echo "BUILDPLATFORM = $BUILDPLATFORM"
RUN echo "TARGETPLATFORM = $TARGETPLATFORM"

# We need the target arch triplet in both Debian and rust flavor
RUN echo "DEBIAN_ARCH_TRIPLET='$(dpkg-architecture -A${TARGETPLATFORM#linux/} -qDEB_TARGET_MULTIARCH)'" >>/root/dynenv
RUN . /root/dynenv && \
    echo "RUST_ARCH_TRIPLET='$(echo "$DEBIAN_ARCH_TRIPLET" | sed -E 's/-linux-/-unknown&/')'" >>/root/dynenv
RUN cat /root/dynenv

WORKDIR /root

# Copy source tree
COPY . .

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y clang cmake protobuf-compiler

# Prepare a local mirror for duniter-polkadot-sdk and force cargo to use it.
RUN chmod +x scripts/prepare_local_polkadot_sdk.sh scripts/cargo_with_vendor.sh && \
    ./scripts/prepare_local_polkadot_sdk.sh Cargo.lock duniter-polkadot-sdk.git

# build duniter
ARG debug=0
RUN if [ "$debug" = 0 ]; then \
        echo "CARGO_OPTIONS=--release" >>/root/dynenv && \
        echo "TARGET_FOLDER=release" >>/root/dynenv; \
    else \
        echo "TARGET_FOLDER=debug" >>/root/dynenv; \
    fi

# Configure cross-build environment if needed to be
RUN set -x && \
    if [ "$TARGETPLATFORM" != "$BUILDPLATFORM" ]; then \
        . /root/dynenv && \
        apt install -y gcc-$DEBIAN_ARCH_TRIPLET binutils-$DEBIAN_ARCH_TRIPLET && \
        rustup target add "$RUST_ARCH_TRIPLET" && \
        : https://github.com/rust-lang/cargo/issues/4133 && \
        echo "RUSTFLAGS='-C linker=$DEBIAN_ARCH_TRIPLET-gcc'; export RUSTFLAGS" >>/root/dynenv; \
    fi

# Build
ARG chain="gdev"
RUN set -x && \
    cat /root/dynenv && \
    . /root/dynenv && \
    ./scripts/cargo_with_vendor.sh build --locked $CARGO_OPTIONS --no-default-features $BENCH_OPTIONS --features ${chain},embed --target "$RUST_ARCH_TRIPLET" && \
    ./scripts/cargo_with_vendor.sh build --locked $CARGO_OPTIONS --target "$RUST_ARCH_TRIPLET" --package distance-oracle && \
    mkdir -p build && \
    mv target/$RUST_ARCH_TRIPLET/$TARGET_FOLDER/duniter build/ && \
    mv target/$RUST_ARCH_TRIPLET/$TARGET_FOLDER/distance-oracle build/

# Run tests if requested, except when cross-building
ARG cucumber=0
RUN if [ "$cucumber" != 0 ] && [ "$TARGETPLATFORM" = "$BUILDPLATFORM" ]; then \
        ./scripts/cargo_with_vendor.sh ta && \
        ./scripts/cargo_with_vendor.sh test --workspace --exclude duniter-end2end-tests --exclude duniter-live-tests --features=runtime-benchmarks,constant-fees && \
        cd target/debug/deps/ && \
        rm cucumber_tests-*.d && \
        mv cucumber_tests* ../../../build/duniter-cucumber; \
    fi

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM target

LABEL maintainer="Gilles Filippini <gilles.filippini@pini.fr>"
LABEL version="0.0.0"
LABEL description="Crypto-currency software (based on Polkadot SDK framework) to operate Ğ1 libre currency"

# Required certificates for RPC connections
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates
RUN update-ca-certificates
RUN apt-get clean && rm -rf /var/lib/apt/lists/*

RUN adduser --home /var/lib/duniter duniter

# Configuration
# rpc, p2p, telemetry
EXPOSE 9944 30333 9615
VOLUME /var/lib/duniter
ENTRYPOINT ["docker-entrypoint"]
USER duniter

# Install
COPY --from=build /root/build /usr/local/bin/
COPY --from=build /root/dynenv /var/lib/duniter
COPY docker/docker-entrypoint /usr/local/bin/
COPY docker/docker-distance-entrypoint /usr/local/bin/

# Debug
RUN cat /var/lib/duniter/dynenv
