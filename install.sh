#!/usr/bin/env bash

echo "===[ Running tests ]==="
cargo test

echo "===[ Building ]==="
cargo build --release

echo "===[ Installing ]==="
cp -v target/release/rendezvous-coach "$HOME"/bin