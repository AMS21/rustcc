#!/usr/bin/env bash

set -euo pipefail

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$SCRIPT_DIR/.."

echo "Updating test baselines..."

cargo run --bin test-driver -- --directory rustcc/tests --update-baseline
