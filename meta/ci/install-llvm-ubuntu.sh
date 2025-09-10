#!/usr/bin/env bash

set -euo pipefail

wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 20 all

echo "LLVM_SYS_201_PREFIX=$(llvm-config-20 --prefix)" >> $GITHUB_ENV

rm -rf llvm.sh