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

cargo +nightly fuzz run fuzz_compile -- -only_ascii=1 -max_len=$MAX_LEN -jobs=$JOBS
