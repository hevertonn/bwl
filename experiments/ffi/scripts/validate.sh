#!/usr/bin/env bash
set -uo pipefail
cd "$(dirname "$0")/.."
mkdir -p out
report="$PWD/out/report.md"
printf '# FFI feasibility probe\n\n| Test | Result | Notes |\n|---|---|---|\n' > "$report"
record() { printf '| %s | %s | %s |\n' "$1" "$2" "$3" | tee -a "$report"; }
echo "Rust: $(rustc --version)"
echo "Clang: $(clang --version | head -1)"
echo "Bend: $(bend version 2>&1 | head -3)"
# Baseline only: C is not Bend.
if clang -std=c11 -O3 -c fixture.c -o out/fixture.o && ar rcs out/libfixture.a out/fixture.o && (cd rust && RUSTFLAGS="-L native=$PWD/../out -l static=fixture" cargo run --release); then
  record 'Rust/C ABI: scalars, struct, buffer, callback, reference timings' 'PASS' 'C fixture only; not proof of Bend FFI'
else
  record 'Rust/C ABI reference' 'FAIL' 'Build or assertions failed'
  exit 1
fi
# Compile real Bend code to C, then a native executable.
if bend bend/main.bend --check-only > out/bend-check.log 2>&1; then
  record 'Bend typecheck' 'PASS' 'Bend 2 syntax'
else
  record 'Bend typecheck' 'FAIL' 'See bend-check.log'
  cat out/bend-check.log
  exit 1
fi
if bend bend/main.bend -o out/bend-generated.c > out/bend-c.log 2>&1; then
  record 'Bend to generated C' 'PASS' 'Artifact available for ABI inspection'
else
  record 'Bend to generated C' 'FAIL' 'See bend-c.log'
  cat out/bend-c.log
  exit 1
fi
if bend bend/main.bend -o out/bend-native > out/bend-native-build.log 2>&1; then
  actual="$(./out/bend-native 2>&1)" || { record 'Bend native execution' 'FAIL' 'Binary exited nonzero'; exit 1; }
  if [[ "$actual" == *42* ]]; then
    record 'Bend native output' 'PASS' 'Computed 6 times 7 = 42'
  else
    record 'Bend native output' 'FAIL' 'Expected 42'; exit 1
  fi
else
  record 'Bend native build' 'FAIL' 'See bend-native-build.log'
  cat out/bend-native-build.log
  exit 1
fi
# Discovery, not a claim that arbitrary Bend functions can be linked.
if clang -std=c11 -O2 -c out/bend-generated.c -o out/bend-generated.o >out/bend-object.log 2>&1; then
  nm -g --defined-only out/bend-generated.o > out/exports.txt || true
  record 'Bend-generated C object' 'PASS' 'Exported symbol inventory in exports.txt'
else
  record 'Bend-generated C object' 'FAIL' 'See bend-object.log'
fi
record 'Rust -> arbitrary Bend function' 'NOT VALIDATED' 'Requires stable exported ABI and runtime lifecycle contract'
record 'Bend -> Rust callback' 'NOT VALIDATED' 'Requires supported Bend foreign imports and callback ABI'
record 'Bend structs, buffers, ownership' 'NOT VALIDATED' 'C fixture confirms Rust ABI mechanics only'
record 'Per-call Bend FFI latency' 'NOT VALIDATED' 'Cannot measure without repeated real Bend foreign calls'
echo "Report: $report"
