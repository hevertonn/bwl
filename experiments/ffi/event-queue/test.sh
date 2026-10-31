#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p out
cargo build --manifest-path rust/Cargo.toml --release
clang -O3 -std=c11 -pthread bench.c rust/target/release/libbwl_event_queue.a -ldl -lm -o out/bench
./out/bench | tee out/bench-results.txt
