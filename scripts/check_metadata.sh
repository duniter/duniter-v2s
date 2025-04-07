#!/bin/sh

cargo install subxt-cli
cargo build
target/debug/duniter --dev&
sleep 20
subxt metadata -f bytes > resources/new_metadata.scale
kill $!

if cmp -s resources/new_metadata.scale resources/metadata.scale; then
    exit 0
else
    echo "Metadata file needs to be generated. How to do it? $HOME/.cargo/bin/subxt metadata -f bytes > resources/metadata.scale"
    if [ "$1" = "--update" ]; then
      mv resources/new_metadata.scale resources/metadata.scale
      echo "Metadata file updated automatically (--update option detected)"
    fi
    exit 1
fi
