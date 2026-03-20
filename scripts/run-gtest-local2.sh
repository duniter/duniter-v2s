#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  scripts/run-gtest-local2.sh alice [--node-key-file PATH] [-- <duniter args...>]
  scripts/run-gtest-local2.sh alice-peer-id [--node-key-file PATH]
  scripts/run-gtest-local2.sh bob --alice-peer-id PEER_ID [-- <duniter args...>]

This helper starts the 2-authority `gtest_local2` chain with real consensus:
  - Alice listens on p2p port 30333 and RPC port 9944
  - Bob listens on p2p port 30334 and RPC port 9945
  - Alice uses a persistent node key file so her peer ID is deterministic

Examples:
  scripts/run-gtest-local2.sh alice
  scripts/run-gtest-local2.sh alice-peer-id
  scripts/run-gtest-local2.sh bob --alice-peer-id 12D3KooW...
  scripts/run-gtest-local2.sh alice -- -lruntime=debug
EOF
}

ROLE=""
ALICE_PEER_ID=""
NODE_ARGS=()
NODE_KEY_FILE="${TMPDIR:-/tmp}/gtest_local2_alice.nodekey"

while (($# > 0)); do
    case "$1" in
        alice|alice-peer-id|bob)
            if [ -n "$ROLE" ]; then
                echo "Role already set to '$ROLE'." >&2
                usage
                exit 1
            fi
            ROLE="$1"
            shift
            ;;
        --alice-peer-id)
            if (($# < 2)); then
                echo "Missing value for --alice-peer-id." >&2
                usage
                exit 1
            fi
            ALICE_PEER_ID="$2"
            shift 2
            ;;
        --node-key-file)
            if (($# < 2)); then
                echo "Missing value for --node-key-file." >&2
                usage
                exit 1
            fi
            NODE_KEY_FILE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            NODE_ARGS+=("$@")
            break
            ;;
        *)
            NODE_ARGS+=("$1")
            shift
            ;;
    esac
done

if [ -z "$ROLE" ]; then
    echo "Missing role: expected one of alice, alice-peer-id, bob." >&2
    usage
    exit 1
fi

if ((${#NODE_ARGS[@]} > 0)); then
    for arg in "${NODE_ARGS[@]}"; do
        case "$arg" in
            --chain|--chain=*|--validator|--alice|--bob|--tmp|--port|--port=*|--rpc-port|--rpc-port=*|--prometheus-port|--prometheus-port=*|--bootnodes|--bootnodes=*|--listen-addr|--listen-addr=*|--node-key-file|--node-key-file=*|--unsafe-force-node-key-generation|--sealing|--sealing=*)
                echo "Do not pass '$arg': this script already sets the required chain and networking arguments." >&2
                exit 1
                ;;
        esac
    done
fi

ensure_alice_node_key() {
    if [ ! -f "$NODE_KEY_FILE" ]; then
        mkdir -p "$(dirname "$NODE_KEY_FILE")"
        cargo run -p duniter --no-default-features --features gtest,fast -- \
            key generate-node-key --chain gtest_local2 --file "$NODE_KEY_FILE"
    fi
}

inspect_alice_peer_id() {
    cargo run -p duniter --no-default-features --features gtest,fast -- \
        key inspect-node-key --file "$NODE_KEY_FILE"
}

append_node_args() {
    if ((${#NODE_ARGS[@]} > 0)); then
        CMD+=("${NODE_ARGS[@]}")
    fi
}

case "$ROLE" in
    alice-peer-id)
        ensure_alice_node_key
        inspect_alice_peer_id
        ;;
    alice)
        ensure_alice_node_key
        CMD=(
            cargo run -p duniter --no-default-features --features gtest,fast --
            --chain gtest_local2
            --validator
            --alice
            --unsafe-force-node-key-generation
            --node-key-file "$NODE_KEY_FILE"
            --listen-addr /ip4/127.0.0.1/tcp/30333
            --tmp
        )
        append_node_args
        exec "${CMD[@]}"
        ;;
    bob)
        if [ -z "$ALICE_PEER_ID" ]; then
            echo "Missing --alice-peer-id for Bob." >&2
            usage
            exit 1
        fi
        CMD=(
            cargo run -p duniter --no-default-features --features gtest,fast --
            --chain gtest_local2
            --validator
            --bob
            --unsafe-force-node-key-generation
            --port 30334
            --rpc-port 9945
            --prometheus-port 9616
            --bootnodes "/ip4/127.0.0.1/tcp/30333/p2p/${ALICE_PEER_ID}"
            --tmp
        )
        append_node_args
        exec "${CMD[@]}"
        ;;
esac
