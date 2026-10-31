#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

mkdir -p out
cargo build --release --manifest-path rust/Cargo.toml
bend main.bend -o out/generated.c

clang -O2 -pthread out/generated.c \
  rust/target/release/libbwl_bend_consumer_bench.a \
  -ldl -lm -o out/bench

./out/bench | tee out/results.txt
grep -q 'BEND_CONSUMER_RESULT events=100000 invalid=0 remaining=0' out/results.txt
