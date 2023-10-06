# Workaround for https://github.com/containers/buildah/issues/4742
FROM debian:bullseye-slim as target
#ARG TARGETPLATFORM

#RUN apt-get update
#RUN apt-get install ca-certificates curl gnupg
#RUN install -m 0755 -d /etc/apt/keyrings
#RUN curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
#RUN chmod a+r /etc/apt/keyrings/docker.gpg
#RUN echo \
#      "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
#      "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
#      tee /etc/apt/sources.list.d/docker.list > /dev/null
#
#RUN apt-get update
#RUN apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
RUN docker run --entrypoint /bin/sh hello-world echo FromDocker

# We need the target arch triplet in both Debian and rust flavor
RUN echo "DEBIAN_ARCH_TRIPLET='$(dpkg-architecture -A${TARGETPLATFORM#linux/} -qDEB_TARGET_MULTIARCH)'" >>/root/dynenv
RUN . /root/dynenv && \
    echo "RUST_ARCH_TRIPLET='$(echo "$DEBIAN_ARCH_TRIPLET" | sed -E 's/-linux-/-unknown&/')'" >>/root/dynenv

WORKDIR /root

# Copy source tree
COPY . .

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y clang cmake protobuf-compiler

# build duniter
ARG debug=0
RUN if [ "$debug" = 0 ]; then \
        echo "CARGO_OPTIONS=--release" >>/root/dynenv && \
        echo "TARGET_FOLDER=release" >>/root/dynenv; \
    else \
        echo "TARGET_FOLDER=debug" >>/root/dynenv; \
    fi

## Configure cross-build environment if need be
#RUN set -x && \
#    if [ "$TARGETPLATFORM" != "$BUILDPLATFORM" ]; then \
#        . /root/dynenv && \
#        apt install -y gcc-$DEBIAN_ARCH_TRIPLET binutils-$DEBIAN_ARCH_TRIPLET && \
#        rustup target add "$RUST_ARCH_TRIPLET" && \
#        : https://github.com/rust-lang/cargo/issues/4133 && \
#        echo "RUSTFLAGS='-C linker=$DEBIAN_ARCH_TRIPLET-gcc'; export RUSTFLAGS" >>/root/dynenv; \
#    fi

# Build
RUN set -x && \
    cargo xtask release-runtime 600

## Run tests if requested, expted when cross-building
#ARG cucumber=0
#RUN if [ "$cucumber" != 0 ] && [ "$TARGETPLATFORM" = "$BUILDPLATFORM" ]; then \
#        cargo test --workspace --exclude duniter-end2end-tests --exclude duniter-live-tests && \
#        cargo cucumber -i account_creation* && \
#        cargo cucumber -i certification* && \
#        cargo cucumber -i identity_creation* && \
#        cargo cucumber -i monetary_mass* && \
#        cargo cucumber -i oneshot_account* && \
#        cargo cucumber -i transfer_all* && \
#        cd target/debug/deps/ && \
#        rm cucumber_tests-*.d && \
#        mv cucumber_tests* ../../../build/duniter-cucumber; \
#    fi

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM target

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
COPY --from=build /root/build /usr/local/bin/
COPY docker/docker-entrypoint /usr/local/bin/
