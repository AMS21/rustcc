#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/.."

# Ensure that all required tools are installed
"$SCRIPT_DIR/fuzz-setup.sh"

JOBS=$(($(nproc) / 2))

echo "Setting up baseline corpus..."

shopt -s globstar nullglob
SEED_DIRS=(crates/rustcc/tests/input/**/)
DICT="fuzz/dictionaries/c.dict"

for i in 2 4 8 16 32 64 128 256; do
    echo "Generating corpus with maximum input length $i..."

    cargo +nightly fuzz run fuzz_compile --release --debug-assertions --jobs=$JOBS -- \
        -dict=$DICT -max_len=$i -runs=1000000 fuzz/corpus/fuzz_compile "${SEED_DIRS[@]}"

    echo "Successfully generated corpus with maximum input length $i."
done

echo "Successfully set up baseline corpus."
