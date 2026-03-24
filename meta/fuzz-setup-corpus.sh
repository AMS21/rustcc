#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/.."

# Ensure that all required tools are installed
"$SCRIPT_DIR/fuzz-setup.sh"

JOBS=$(($(nproc) / 2))

echo "Setting up baseline corpus..."
echo "Using $JOBS parallel jobs for fuzzing."
echo "This might take a while, so please be patient."

DICT="fuzz/dictionaries/c.dict"
SEED_DIR="crates/rustcc/tests/ui/"

for i in 2 4 8 16 32 64 128 256 512 1024 2048; do
    echo "Generating corpus with maximum input length $i..."

    RUSTFLAGS="-Cdebuginfo=1 -Cforce-frame-pointers" cargo \
        +nightly fuzz run fuzz_compile \
        fuzz/corpus/fuzz_compile "${SEED_DIR}" \
        --release --debug-assertions -- \
        -dict=$DICT -max_len=$i -runs=1000000 \
        -jobs=$JOBS -workers=$JOBS -close_fd_mask=1

    echo "Successfully generated corpus with maximum input length $i."
done

echo "Successfully set up baseline corpus."
