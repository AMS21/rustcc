#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/.."

# Ensure that all required tools are installed
"$SCRIPT_DIR/fuzz-setup.sh"

# Move the old corpus to a backup directory
echo "Moving old corpus to backup directory..."
mkdir -p fuzz/old_corpus
mv fuzz/corpus/fuzz_compile/* fuzz/old_corpus/

# Minimize the corpus
echo "Minimizing the corpus..."
cargo +nightly fuzz run fuzz_compile -- -merge=1 fuzz/old_corpus
echo "Minimization complete."

# Cleanup
mv fuzz/old_corpus/* fuzz/corpus/fuzz_compile/
rm -rf fuzz/old_corpus
