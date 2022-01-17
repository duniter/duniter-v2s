# ------------------------------------------------------------------------------
# Build Stage
# ------------------------------------------------------------------------------

# Building for Debian buster because we need the binary to be compatible
# with the image paritytech/ci-linux:production (currently based on
# debian:buster-slim) used by the gitlab CI
FROM rust:1-buster as build
WORKDIR /root

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y \
        clang

# Copy source tree
COPY . .

# build duniter
ARG threads=1
RUN test -x build/duniter || \
    ( \
        CARGO_PROFILE_RELEASE_LTO="true" \
            cargo build --release -j $threads && \
        mkdir -p build && \
        mv target/release/duniter build/ \
    )

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM debian:buster-slim

LABEL maintainer="Gilles Filippini <gilles.filippini@pini.fr>"
LABEL version="0.0.0"
LABEL description="Crypto-currency software (based on Substrate framework) to operate Ğ1 libre currency"

# Intall
COPY --from=build /root/build/duniter /usr/local/bin/duniter
COPY docker/docker-entrypoint /usr/local/bin/

# Configuration
# rpc, rpc-ws, p2p, telemetry
EXPOSE 9933 9944 30333 9615
VOLUME /var/lib/duniter
ENTRYPOINT ["docker-entrypoint"]
