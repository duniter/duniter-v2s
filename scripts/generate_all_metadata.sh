#!/bin/bash
set -e

for chain in g1 gtest gdev
do
  cargo build --release --no-default-features --features $chain
  ./target/release/duniter --dev &
  NODE_PID=$!
  sleep 20
  $HOME/.cargo/bin/subxt metadata -f bytes > resources/${chain}_metadata.scale
  kill $NODE_PID
  wait $NODE_PID 2>/dev/null || true
done
