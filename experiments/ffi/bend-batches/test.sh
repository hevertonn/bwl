#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p out
cargo build --release --manifest-path rust/Cargo.toml
bend main.bend -o out/generated.c
clang -O2 -pthread out/generated.c \
  rust/target/release/libbwl_bend_batches.a -ldl -lm -o out/bench

: > out/measurements.txt
for pause in 0 1000; do
  for batch in 16 64 256; do
    echo "SCENARIO batch=$batch pause_us=$pause" | tee -a out/measurements.txt
    BWL_BATCH="$batch" BWL_PAUSE_US="$pause" \
      ./out/bench | tee -a out/measurements.txt
  done
done
