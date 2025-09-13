#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/.."

# Ensure that all required tools are installed
"$SCRIPT_DIR/fuzz-setup.sh"

# Check if a corpus already exists and if not, generate one
if [ ! -d "fuzz/corpus" ]; then
    echo "No corpus found, generating one..."

    "$SCRIPT_DIR/fuzz-setup-corpus.sh"
fi

MAX_LEN=512
JOBS=$(($(nproc) / 2))

# Fuzz the compiler
echo "Fuzzing the compiler..."

# Use dictionary and merge seeds from tests as additional corpus directory
DICT="fuzz/dictionaries/c.dict"
shopt -s globstar nullglob
SEED_DIRS=(crates/rustcc/tests/input/**/)

cargo +nightly fuzz run fuzz_compile --release --debug-assertions --jobs=$JOBS -- \
    -only_ascii=1 -max_len=$MAX_LEN -close_fd_mask=3 \
    -dict=$DICT fuzz/corpus/fuzz_compile "${SEED_DIRS[@]}"
