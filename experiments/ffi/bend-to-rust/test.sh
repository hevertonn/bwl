#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p out
cargo build --release --manifest-path rust/Cargo.toml
bend main.bend -o out/generated.c
# The Bend-generated native code contains the effect adapter, which calls a
# Rust symbol from a static library. No IPC or separate C library is involved.
clang -O2 -pthread out/generated.c rust/target/release/libbwl_bend_effect_rust.a -ldl -lm -o out/probe
out="$(./out/probe)"
printf '%s\n' "$out"
grep -q 'RUST_EFFECT_CALLED:1' <<< "$out"
grep -q 'BEND_AFTER_RUST' <<< "$out"
echo 'PASS: Bend native effect invoked real Rust staticlib in-process'
