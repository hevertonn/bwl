# Event queue prototype (Rust native)

This is an **ABI baseline**, not yet a Bend consumer. Rust owns an 8192-element bounded queue. The C benchmark driver invokes its exported ABI and measures sequential enqueue + batched drain (256 events), event age at consumption (p50/p99), throughput and a condition-variable idle wait. It checks sequence integrity and drops.

These are **same-thread, synthetic measurements**. The event creation timestamp and drain happen in the same test driver; they do not model hardware, compositor frame latency, competing threads or Bend runtime overhead. CI results are noisy. A successful run does not validate end-to-end Bend ↔ Rust throughput.

Next requirement: implement Bend-side batched dequeue and measure with identical event streams.
