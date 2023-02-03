# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------

# Building for Debian buster because we need the binary to be compatible
# with the image paritytech/ci-linux:production (currently based on
# debian:buster-slim) used by the gitlab CI
FROM rust:1-buster as build
WORKDIR /root

# Copy source tree
COPY . .

RUN test -x build/duniter || \
    ( \
        apt-get update && \
        DEBIAN_FRONTEND=noninteractive apt-get install -y clang cmake protobuf-compiler \
    )

# build duniter
ARG threads=1
RUN test -x build/duniter || \
    ( \
        CARGO_PROFILE_RELEASE_LTO="true" \
            cargo build --release -j $threads && \
        mkdir -p build && \
        mv target/release/duniter build/ \
    )

# Create fake duniter-cucumber if is not exist
# The goal is to avoid error later, this binary is optional
RUN test -x build/duniter-cucumber || \
    ( \
        mkdir -p build && \
        touch build/duniter-cucumber \
    )

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:buster-slim

LABEL maintainer="Gilles Filippini <gilles.filippini@pini.fr>"
LABEL version="0.0.0"
LABEL description="Crypto-currency software (based on Substrate framework) to operate Ğ1 libre currency"

RUN adduser --home /var/lib/duniter duniter

# Configuration
# rpc, rpc-ws, p2p, telemetry
EXPOSE 9933 9944 30333 9615
VOLUME /var/lib/duniter
ENTRYPOINT ["docker-entrypoint"]
USER duniter

# Intall
COPY --from=build /root/build/duniter /usr/local/bin/duniter
COPY --from=build /root/build/duniter-cucumber /usr/local/bin/duniter-cucumber
COPY docker/docker-entrypoint /usr/local/bin/
