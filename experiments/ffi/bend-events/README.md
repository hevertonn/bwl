# Bend-side event processing (real value transfer)

The Rust producer generates 20,000 ordered U32 event identifiers on a separate thread and pushes them through a bounded 1,024-item queue. Every `Probe.next()` effect transfers an actual `U32` to Bend. **Bend itself adds the values**, and `Probe.finish(checksum)` returns the computed result to Rust for independent validation.

Measures throughput and queue residence latency P50/P99, plus maximum queue depth and loss/integrity via arithmetic checksum. This is an intentionally unbatched, worst-case **one-effect-per-event** reference against our earlier batched native queue.

The event payload is currently a U32 identifier rather than a full Wayland structure. Queue timing is measured at the Rust dequeue boundary, **before** Bend processes the returned value; therefore these latencies are not end-to-end Bend handling latencies. CPU load, frame deadlines, and multiple producers are not yet simulated. Treat CI timings as indicative, not stable platform characteristics.
