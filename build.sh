#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$SCRIPT_DIR/koko/target/release"

echo "Building koko..."
cd "$SCRIPT_DIR/koko"
cargo build --release

EXPORT_LINE="export PATH=\"\$PATH:$BIN_DIR\""

if grep -qF "$BIN_DIR" ~/.bashrc; then
    echo "PATH already contains $BIN_DIR, skipping."
else
    echo "$EXPORT_LINE" >> ~/.bashrc
    echo "Added $BIN_DIR to PATH in ~/.bashrc"
fi

echo "Done. Run 'source ~/.bashrc' or open a new terminal to use koko."
