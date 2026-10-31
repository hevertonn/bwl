#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
mkdir -p out

cargo build --release --manifest-path rust/Cargo.toml
bend main.bend -o out/generated.c

clang -O2 -pthread out/generated.c \
  rust/target/release/libbwl_bend_events.a \
  -ldl -lm -o out/probe

./out/probe | tee out/results.txt
grep -q 'BEND_VALUE_RESULT events=20000 checksum=200010000 expected=200010000 queue_remaining=0' out/results.txt
