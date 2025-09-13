#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/.."

# Ensure that all required tools are installed
"$SCRIPT_DIR/fuzz-setup.sh"

# Minimize the corpus
echo "Minimizing the corpus..."
shopt -s globstar nullglob
SEED_DIRS=(crates/rustcc/tests/input/**/)

cargo +nightly fuzz cmin fuzz_compile --release --debug-assertions -- \
    fuzz/corpus fuzz/corpus/fuzz_compile "${SEED_DIRS[@]}"
echo "Minimization complete."
