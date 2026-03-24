#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/../"

echo "Blessing ui tests..."

RUSTCC_BLESS=1 cargo test -p rustcc-compiler --test ui -- --nocapture
