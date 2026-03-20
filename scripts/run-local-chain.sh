#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/run-local-chain.sh [--runtime gdev|gtest|g1] [--technical-committee-members N] [-- <duniter args...>]

This helper always runs with local-chain defaults:
  --validator --unsafe-force-node-key-generation --sealing manual --tmp

Examples:
  scripts/run-local-chain.sh
  scripts/run-local-chain.sh --runtime gtest
  scripts/run-local-chain.sh --runtime gtest --technical-committee-members 3
  scripts/run-local-chain.sh --runtime g1 -- -lruntime=debug
EOF
}

RUNTIME="gdev"
TECHNICAL_COMMITTEE_MEMBERS=""
NODE_ARGS=()
FEATURES=""

while (($# > 0)); do
    case "$1" in
        --runtime)
            if (($# < 2)); then
                echo "Missing value for --runtime." >&2
                usage
                exit 1
            fi
            RUNTIME="$2"
            shift 2
            ;;
        --technical-committee-members)
            if (($# < 2)); then
                echo "Missing value for --technical-committee-members." >&2
                usage
                exit 1
            fi
            TECHNICAL_COMMITTEE_MEMBERS="$2"
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

case "$RUNTIME" in
    gdev|gtest|g1) ;;
    *)
        echo "Invalid runtime '$RUNTIME'. Must be one of: gdev, gtest, g1." >&2
        exit 1
        ;;
esac

case "$RUNTIME" in
    g1|gtest)
        FEATURES="${RUNTIME},fast"
        ;;
    gdev)
        FEATURES="$RUNTIME"
        ;;
esac

if [ -n "$TECHNICAL_COMMITTEE_MEMBERS" ]; then
    if ! [[ "$TECHNICAL_COMMITTEE_MEMBERS" =~ ^[0-9]+$ ]]; then
        echo "--technical-committee-members must be a positive integer." >&2
        exit 1
    fi
    if [ "$TECHNICAL_COMMITTEE_MEMBERS" -lt 1 ]; then
        echo "--technical-committee-members must be >= 1." >&2
        exit 1
    fi
    export DUNITER_LOCAL_TECHNICAL_COMMITTEE_MEMBERS="$TECHNICAL_COMMITTEE_MEMBERS"
fi

for arg in "${NODE_ARGS[@]}"; do
    case "$arg" in
        --validator|--unsafe-force-node-key-generation|--tmp|--sealing|--sealing=*)
            echo "Do not pass '$arg': this script already enforces --validator --unsafe-force-node-key-generation --sealing manual --tmp." >&2
            exit 1
            ;;
    esac
done

exec cargo run -p duniter --no-default-features --features "$FEATURES" -- \
    --chain "${RUNTIME}_local" \
    --validator \
    --unsafe-force-node-key-generation \
    --sealing manual \
    --tmp \
    "${NODE_ARGS[@]}"
