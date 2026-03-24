#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/.."

# Ensure that all required tools are installed
"$SCRIPT_DIR/fuzz-setup.sh"

# Minimize the corpus
RUSTFLAGS="-Cdebuginfo=1 -Cforce-frame-pointers" cargo \
    +nightly fuzz cmin fuzz_compile --release --debug-assertions

echo "Minimization complete."
