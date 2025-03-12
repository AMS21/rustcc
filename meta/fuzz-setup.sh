#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/.."

echo "Installing required tools..."

rustup install nightly
cargo install cargo-fuzz

echo "Successfully installed required tools."
