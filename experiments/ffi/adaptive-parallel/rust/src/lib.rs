//! Multi-producer Rust backend, deadline-aware native batches, Bend checksum.
//! Pure Bend parallel execution is a separate question from concurrent Rust producers.
use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

const PRODUCERS: u32 = 4;
const PER_PRODUCER: u32 = 4096;
const TOTAL: u32 = PRODUCERS * PER_PRODUCER;
const CAPACITY: usize = 2048;
const BATCH_MAX: usize = 256;
const DEADLINE: Duration = Duration::from_micros(250);

struct Event {
    value: u32,
    time: Instant,
    urgent: bool,
}

struct Queue {
    priority: VecDeque<Event>,
    normal: VecDeque<Event>,
    complete: u32,
    consumed: u32,
    latency: Vec<u64>,
    maximum: usize,
    urgent_count: u32,
    urgent_latency: Vec<u64>,
}

struct Shared {
    queue: Mutex<Queue>,
    ready: Condvar,
    space: Condvar,
    start: Mutex<Option<Instant>>,
}

static STATE: OnceLock<Shared> = OnceLock::new();

fn shared() -> &'static Shared {
    STATE.get_or_init(|| Shared {
        queue: Mutex::new(Queue {
            priority: VecDeque::new(),
            normal: VecDeque::new(),
            complete: 0,
            consumed: 0,
            latency: Vec::with_capacity(TOTAL as usize),
            maximum: 0,
            urgent_count: 0,
            urgent_latency: Vec::new(),
        }),
        ready: Condvar::new(),
        space: Condvar::new(),
        start: Mutex::new(None),
    })
}

fn occupancy(q: &Queue) -> usize {
    q.priority.len() + q.normal.len()
}

#[no_mangle]
pub extern "C" fn bwl_adaptive_start() {
    let s = shared();
    *s.start.lock().unwrap() = Some(Instant::now());

    for producer_id in 0..PRODUCERS {
        thread::spawn(move || {
            let s = shared();
            for offset in 1..=PER_PRODUCER {
                let value = producer_id * PER_PRODUCER + offset;
                let urgent = offset % 32 == 0;
                let mut q = s.queue.lock().unwrap();
                while occupancy(&q) == CAPACITY {
                    q = s.space.wait(q).unwrap();
                }

                let event = Event { value, time: Instant::now(), urgent };
                if urgent { q.priority.push_back(event); }
                else { q.normal.push_back(event); }
                q.maximum = q.maximum.max(occupancy(&q));
                s.ready.notify_one();
                drop(q);

                // Simulate mixed bursts and interleaved short periods.
                if offset % 64 == 0 {
                    thread::sleep(Duration::from_micros(100));
                }
            }

            let mut q = s.queue.lock().unwrap();
            q.complete += 1;
            s.ready.notify_all();
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn bwl_adaptive_take(out: *mut u32, cap: u32) -> u32 {
    if out.is_null() { return 0; }
    let s = shared();
    let mut q = s.queue.lock().unwrap();
    let start = Instant::now();

    while occupancy(&q) < BATCH_MAX
        && q.priority.is_empty()
        && q.complete < PRODUCERS
        && start.elapsed() < DEADLINE
    {
        let left = DEADLINE.saturating_sub(start.elapsed());
        let (next, _) = s.ready.wait_timeout(q, left).unwrap();
        q = next;
    }

    let n = occupancy(&q).min(cap as usize).min(BATCH_MAX);
    for i in 0..n {
        let event = q.priority.pop_front().or_else(|| q.normal.pop_front()).unwrap();
        let latency = event.time.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        if event.urgent {
            q.urgent_count += 1;
            q.urgent_latency.push(latency);
        }
        q.latency.push(latency);
        q.consumed += 1;
        out.add(i).write(event.value);
    }
    s.space.notify_all();
    n as u32
}

#[no_mangle]
pub extern "C" fn bwl_adaptive_finish(sum: u32) {
    let s = shared();
    let q = s.queue.lock().unwrap();
    let elapsed = s.start.lock().unwrap().unwrap().elapsed();
    let expected = (TOTAL as u64 * (TOTAL as u64 + 1) / 2) as u32;
    let mut lat = q.latency.clone();
    lat.sort_unstable();
    let mut urgent = q.urgent_latency.clone();
    urgent.sort_unstable();

    let pct = |v: &[u64], n: usize| v.get(v.len().saturating_mul(n) / 100).copied().unwrap_or(0);
    println!(
        "ADAPTIVE_RESULT events={} checksum={} expected={} remaining={} producers_done={} high_water={} urgent={} duration_ms={:.3} rate_per_s={:.0} p50_ns={} p99_ns={} urgent_p99_ns={}",
        q.consumed, sum, expected, occupancy(&q), q.complete, q.maximum,
        q.urgent_count, elapsed.as_secs_f64() * 1000.0,
        q.consumed as f64 / elapsed.as_secs_f64(),
        pct(&lat, 50), pct(&lat, 99), pct(&urgent, 99)
    );

    if q.consumed != TOTAL || sum != expected || occupancy(&q) != 0
        || q.complete != PRODUCERS || q.urgent_count != TOTAL / 32
    {
        std::process::abort();
    }
}
