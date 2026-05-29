#!/usr/bin/env bash
set -e

TARGET_DIR=$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['target_directory'])")

echo "===[ Running tests ]==="
cargo test

echo "===[ Building ]==="
cargo build --release

echo "===[ Installing ]==="
cp -v "$TARGET_DIR/release/rendezvous-coach" "$HOME/bin/"
find "$TARGET_DIR/release" -maxdepth 1 -name "*.so*" -exec cp -Pv {} "$HOME/bin/" \;
