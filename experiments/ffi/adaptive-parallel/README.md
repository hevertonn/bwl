# Adaptive multi-producer experiment

Four concurrent Rust producers push 16,384 unique U32 identifiers into a bounded queue. Events are marked urgent or normal. The consumer, invoked from real Bend effects, prefers urgent events; it wakes on urgent work, at 256 pending events, or after a 250 µs deadline. Bend receives a `List<U32>`, sums every value itself, and Rust checks the result.

Reports total throughput, queue residence P50/P99, urgent P99 and integrity. Multiple Rust producer threads are real. **Bend IO remains a single event loop**: this benchmark does not establish simultaneous Bend IO workers or parallelism in Bend pure reductions. Bend's compiler/runtime may parallelize eligible pure work, but the simple checksum must not be advertised as proof of that.

Caution: urgency changes global event order. For real Wayland semantics, order-dependent groups (per device/surface, focus transitions) must be serialized or reconciled. There is no actual Wayland server, renderer or presentation deadline in this experiment. The deadline is a batch-coalescing timeout, not a frame deadline.
