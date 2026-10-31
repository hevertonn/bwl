# Real Bend-side batch processing

An independent Rust thread generates 16,384 ordered U32 events; the bounded queue holds 2,048. Rust exposes 16/64/256-event batches to the Bend runtime. The native adapter only builds a Bend `List(U32)`; the **checksum is computed by Bend itself**.

The test runs six combinations: 16, 64 and 256 items per batch, either a saturation workload (no producer delay) or a paced producer (1 ms per 64 events). It checks checksum, event count, FIFO ordering and empty final queue. It reports queue-residence P50/P99 and throughput.

**Limits:** synthetic values, one producer, shared GitHub runner, no Wayland/display/GPU, timing recorded when Rust dequeues (before Bend completes its processing). Therefore these are *not* full input-to-presentation latency measurements. The CI step may require compatibility fixes as Bend's internal C effects API has no stable ABI. No performance conclusion until real CI logs are reviewed.
