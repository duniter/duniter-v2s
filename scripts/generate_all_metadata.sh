#!/usr/bin/env bash
set -euo pipefail

CHAINS=(g1 gtest gdev)
RESOURCES_DIR="$(cd "$(dirname "$0")/../resources" && pwd)"
CARGO_TARGET_DIR="$(cd "$(dirname "$0")/.." && pwd)/target"
NODE_PID=""
RPC_PORT=19944  # non-standard port to avoid conflicts with a running node
RPC_URL="http://127.0.0.1:${RPC_PORT}"
MAX_WAIT=120  # seconds to wait for node startup

cleanup() {
    if [ -n "$NODE_PID" ] && kill -0 "$NODE_PID" 2>/dev/null; then
        echo "Stopping node (PID $NODE_PID)..."
        kill "$NODE_PID" 2>/dev/null
        wait "$NODE_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

# Check for subxt CLI
if ! command -v subxt &>/dev/null; then
    echo "Error: subxt-cli not found. Install it with:"
    echo "  cargo install subxt-cli"
    exit 1
fi

# Ensure port is free
if lsof -i :"$RPC_PORT" -sTCP:LISTEN &>/dev/null 2>&1 || ss -tlnp 2>/dev/null | grep -q ":${RPC_PORT} "; then
    echo "Error: port $RPC_PORT is already in use. Stop any running node first."
    exit 1
fi

wait_for_node() {
    local elapsed=0
    echo "Waiting for node RPC on port $RPC_PORT..."
    while ! curl -s -o /dev/null -w '' "$RPC_URL" 2>/dev/null; do
        if ! kill -0 "$NODE_PID" 2>/dev/null; then
            echo "Error: node process died unexpectedly."
            return 1
        fi
        if [ "$elapsed" -ge "$MAX_WAIT" ]; then
            echo "Error: node did not start within ${MAX_WAIT}s."
            return 1
        fi
        sleep 2
        elapsed=$((elapsed + 2))
    done
    echo "Node is ready (took ~${elapsed}s)."
}

# Bootstrap: if any metadata file is empty or missing, copy a valid one as placeholder
# This solves the chicken-and-egg problem where cargo build fails on empty metadata.
bootstrap_placeholder() {
    local donor=""
    # Find a valid metadata file (non-empty) to use as donor
    for chain in "${CHAINS[@]}"; do
        local f="$RESOURCES_DIR/${chain}_metadata.scale"
        if [ -f "$f" ] && [ -s "$f" ]; then
            donor="$f"
            break
        fi
    done

    for chain in "${CHAINS[@]}"; do
        local f="$RESOURCES_DIR/${chain}_metadata.scale"
        if [ ! -f "$f" ] || [ ! -s "$f" ]; then
            if [ -z "$donor" ]; then
                echo "Error: no valid metadata file found to bootstrap. Cannot continue."
                echo "Manually place a valid .scale file in $RESOURCES_DIR/"
                exit 1
            fi
            echo "Bootstrapping $f with placeholder from $donor"
            cp "$donor" "$f"
        fi
    done
}

bootstrap_placeholder

for chain in "${CHAINS[@]}"; do
    echo ""
    echo "========================================"
    echo " Generating metadata for: $chain"
    echo "========================================"

    echo "Building runtime..."
    cargo build --release --no-default-features --features "$chain"

    echo "Starting node (--dev --tmp)..."
    "$CARGO_TARGET_DIR/release/duniter" --dev --tmp --rpc-port "$RPC_PORT" &
    NODE_PID=$!

    if ! wait_for_node; then
        echo "Failed to start node for $chain, aborting."
        exit 1
    fi

    local_out="$RESOURCES_DIR/${chain}_metadata.scale"
    echo "Extracting metadata to $local_out..."
    subxt metadata --url "ws://127.0.0.1:${RPC_PORT}" -f bytes > "$local_out"

    # Validate the file is non-empty
    if [ ! -s "$local_out" ]; then
        echo "Error: generated metadata file is empty for $chain."
        exit 1
    fi

    echo "Stopping node..."
    kill "$NODE_PID" 2>/dev/null
    wait "$NODE_PID" 2>/dev/null || true
    NODE_PID=""

    echo "Done: $chain ($(wc -c < "$local_out" | tr -d ' ') bytes)"
done

echo ""
echo "All metadata files generated successfully."
