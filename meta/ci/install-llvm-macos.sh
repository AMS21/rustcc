#!/usr/bin/env bash

set -euo pipefail

brew install llvm@20

echo "LLVM_SYS_201_PREFIX=$(llvm-config-20 --prefix)" >> $GITHUB_ENV

rm -rf llvm.sh