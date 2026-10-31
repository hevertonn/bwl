# Rust / Bend 2 FFI feasibility probe

**Isolated experiment:** branch `experiment/rust-bend-ffi`, not intended for merge into `main`.

Run from repository root: `bash experiments/ffi/scripts/validate.sh` (requires Bend 2, clang, Rust, ar, nm).

The CI job tests:
1. Rust/C baseline: u32 arguments and returns, `#[repr(C)]` struct, borrowed byte buffer, C calling Rust callback, and illustrative timings.
2. A **real Bend 2** source: type checking, C emission, native compilation and execution, and exported symbol inspection.
3. Explicitly identifies unproven parts: direct Rust -> arbitrary Bend calls, Bend -> Rust calls, Bend memory ownership and real per-call latency.

**Important:** emitting C does **not** demonstrate a supported FFI. C fixture tests are only a control case. Bend may generate a C `main` and runtime implementation with unstable/private symbols, and the emitted C is not necessarily a linkable library with a stable external API. Do not interpret green CI as an endorsement of embedding Bend's runtime.

Outputs are uploaded as CI artifacts; `report.md` distinguishes PASS/FAIL/NOT VALIDATED. Benchmarks on shared runners are not precise performance measurements.
