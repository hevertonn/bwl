#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p out
cargo build --release --manifest-path rust/Cargo.toml
bend main.bend -o out/generated.c
clang -O2 -pthread out/generated.c \
  rust/target/release/libbwl_adaptive_parallel.a -ldl -lm -o out/bench
./out/bench | tee out/results.txt
grep -q 'ADAPTIVE_RESULT events=16384 checksum=134225920 expected=134225920 remaining=0 producers_done=4' out/results.txt
