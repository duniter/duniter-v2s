#!/usr/bin/env bash
#
# USAGE

# This script is meant to be run on Unix/Linux based systems
set -e

# params
CURRENCY="${1:-gdev}"
GENESIS="${2:-resources/$CURRENCY.json}"

# constants
DUNITER_BINARY="./target/debug/duniter"

# generate raw_chain spec
export DUNITER_GENESIS_CONFIG=$GENESIS
$DUNITER_BINARY build-spec --chain $CURRENCY-gl --raw > resources/$CURRENCY-raw.json
