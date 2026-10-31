# Concurrent producer / Bend-driven consumer

This experiment integrates **actual Bend IO effects** with Rust's native queue:

- A Rust worker generates 100,000 ordered events on its own OS thread.
- The queue is bounded to 8,192 entries and the worker waits for space (backpressure).
- Bend calls `Probe.consume()` 1,024 times, each invoking Rust through a native effect.
- Each call waits efficiently for events, if necessary, and drains up to 256 events.
- Integrity, queue residence latency (P50/P99), throughput, and high-water occupancy are reported.

## Limitations

Bend controls when each batch is consumed, but the event decoding and
per-event accounting currently occur in Rust. This validates real Bend-driven
native calls under concurrency; it **does not** measure per-event processing in
Bend, nor does it simulate an actual Wayland input/rendering pipeline.

The 1,024-call budget exceeds the 391 batches needed for 100,000 events.
Extra calls return immediately after the producer signals completion.
Benchmarks on shared GitHub runners are illustrative; compare repeated runs.
