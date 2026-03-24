#!/bin/bash -eu

cargo fuzz build -O --debug-assertions

# Collect all test cases into a seed corpus
mkdir -p seed_corpus
find crates/rustcc/tests/ui -type f -iname '*.c' -exec sh -c '
  for f do
    h=$(sha256sum "$f" | cut -d" " -f1)
    cp -- "$f" "seed_corpus/$h.c"
  done
' sh {} +
zip -j seed_corpus.zip seed_corpus/*

FUZZ_TARGET_OUTPUT_DIR=fuzz/target/x86_64-unknown-linux-gnu/release
for f in fuzz/fuzz_targets/*.rs
do
    FUZZ_TARGET_NAME=$(basename "${f%.*}")
    cp "${FUZZ_TARGET_OUTPUT_DIR}/${FUZZ_TARGET_NAME}" "${OUT}/"

    # Copy dictionary file
    cp fuzz/dictionaries/c.dict "${OUT}/${FUZZ_TARGET_NAME}.dict"
    # Copy seed corpus
    cp seed_corpus.zip "${OUT}/${FUZZ_TARGET_NAME}_seed_corpus.zip"
done
