FROM paritytech/ci-linux:production

# Set the working directory
WORKDIR /app/

# Copy the toolchain
COPY rust-toolchain.toml ./
COPY scripts/prepare_local_polkadot_sdk.sh scripts/cargo_with_vendor.sh scripts/

# Install toolchain, substrate and cargo-deb with cargo cache
RUN --mount=type=cache,target=/root/.cargo \
    cargo install cargo-deb

# Create a dummy project to cache dependencies
COPY Cargo.toml .
COPY Cargo.lock .
COPY rust-toolchain.toml ./
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    chmod +x scripts/prepare_local_polkadot_sdk.sh scripts/cargo_with_vendor.sh && \
    ./scripts/prepare_local_polkadot_sdk.sh Cargo.lock duniter-polkadot-sdk.git && \
    mkdir src && \
    sed -i '/git = \|version = /!d' Cargo.toml && \
    sed -i 's/false/true/' Cargo.toml && \
    sed -i '1s/^/\[package\]\nname\=\"Dummy\"\n\[dependencies\]\n/' Cargo.toml && \
    echo "fn main() {}" > src/main.rs && \
    ./scripts/cargo_with_vendor.sh build --release && \
    rm -rf src Cargo.lock Cargo.toml

# Copy the entire project
COPY . .

# Build the project and create Debian packages
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    chmod +x scripts/prepare_local_polkadot_sdk.sh scripts/cargo_with_vendor.sh && \
    ./scripts/prepare_local_polkadot_sdk.sh Cargo.lock duniter-polkadot-sdk.git && \
    ./scripts/cargo_with_vendor.sh build --release && \
    ./scripts/cargo_with_vendor.sh deb --no-build -p duniter && \
    cp -r ./target/debian/ ./

# Clean up unnecessary files to reduce image size
RUN rm -rf /app/target/release /root/.cargo/registry
