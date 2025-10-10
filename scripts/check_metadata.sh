#!/bin/sh

cargo install subxt-cli
cargo build

for chain in g1 gtest gdev
do
  cargo build --release --no-default-features --features $chain
  ./target/release/duniter --dev &
  NODE_PID=$!
  sleep 20
  subxt metadata -f bytes > resources/${chain}_new_metadata.scale
  kill $NODE_PID
  wait $NODE_PID 2>/dev/null || true

  if cmp -s resources/${chain}_new_metadata.scale resources/${chain}_metadata.scale; then
      exit 0
  else
      echo "Metadata file needs to be generated. How to do it? scripts/generate_all_metadata.sh"
      exit 1
  fi

done
