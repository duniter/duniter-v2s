FROM paritytech/ci-linux:production

# Set the working directory
WORKDIR /app/

# Copy the toolchain
COPY rust-toolchain.toml ./

# Install toolchain, substrate and cargo-deb with cargo cache
RUN --mount=type=cache,target=/root/.cargo \
    cargo install cargo-deb

# Create a dummy project to cache dependencies
COPY Cargo.toml .
COPY rust-toolchain.toml ./
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cargo/registry \
    mkdir src && \
    sed -i '/git = \|version = /!d' Cargo.toml && \
    sed -i 's/false/true/' Cargo.toml && \
    sed -i '1s/^/\[package\]\nname\=\"Dummy\"\n\[dependencies\]\n/' Cargo.toml && \
    echo "fn main() {}" > src/main.rs && \
    cargo build -Zgit=shallow-deps --release && \
    rm -rf src Cargo.lock Cargo.toml

# Copy the entire project
COPY . .

# Build the project and create Debian packages
RUN --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/root/.cargo/registry \
    cargo build -Zgit=shallow-deps --release && \
    cargo deb --no-build -p duniter && \
    cp -r ./target/debian/ ./

# Clean up unnecessary files to reduce image size
RUN rm -rf /app/target/release /root/.cargo/registry
