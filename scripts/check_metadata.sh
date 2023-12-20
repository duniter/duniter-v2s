#!/bin/sh

cargo install subxt-cli
cargo build
cargo run -- --dev&
sleep 15
subxt metadata -f bytes > resources/new_metadata.scale
kill $!

if cmp -s resources/new_metadata.scale resources/metadata.scale; then
    exit 0
else
    echo "Metadata file needs to be generated. How to do it? $HOME/.cargo/bin/subxt metadata -f bytes > resources/metadata.scale"
    exit 1
fi
