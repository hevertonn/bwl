//! Native event producer + bounded FIFO. The Bend program processes payloads.
use std::{
    collections::VecDeque,
    env,
    sync::{Condvar, Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

const EVENTS: u32 = 16_384;
const CAPACITY: usize = 2048;

struct Event {
    value: u32,
    at: Instant,
}

struct State {
    queue: VecDeque<Event>,
    finished: bool,
    processed: u32,
    previous: u32,
    errors: u32,
    high_water: usize,
    latency: Vec<u64>,
}

struct Shared {
    queue: Mutex<State>,
    available: Condvar,
    space: Condvar,
    start: Mutex<Option<Instant>>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

fn shared() -> &'static Shared {
    SHARED.get_or_init(|| Shared {
        queue: Mutex::new(State {
            queue: VecDeque::with_capacity(CAPACITY),
            finished: false,
            processed: 0,
            previous: 0,
            errors: 0,
            high_water: 0,
            latency: Vec::with_capacity(EVENTS as usize),
        }),
        available: Condvar::new(),
        space: Condvar::new(),
        start: Mutex::new(None),
    })
}

fn batch_size() -> usize {
    env::var("BWL_BATCH").unwrap_or_else(|_| "64".into())
        .parse::<usize>().expect("BWL_BATCH must be an integer")
}

#[no_mangle]
pub extern "C" fn bwl_batches_start() {
    let s = shared();
    *s.start.lock().unwrap() = Some(Instant::now());

    let pause_us = env::var("BWL_PAUSE_US").unwrap_or_default()
        .parse::<u64>().unwrap_or(0);
    thread::spawn(move || {
        for sequence in 1..=EVENTS {
            let s = shared();
            let mut q = s.queue.lock().unwrap();
            while q.queue.len() == CAPACITY {
                q = s.space.wait(q).unwrap();
            }
            q.queue.push_back(Event { value: sequence, at: Instant::now() });
            q.high_water = q.high_water.max(q.queue.len());
            s.available.notify_one();
            drop(q);
            if pause_us > 0 && sequence % 64 == 0 {
                thread::sleep(Duration::from_micros(pause_us));
            }
        }
        let s = shared();
        let mut q = s.queue.lock().unwrap();
        q.finished = true;
        s.available.notify_all();
    });
}

#[no_mangle]
pub unsafe extern "C" fn bwl_batches_take(out: *mut u32, capacity: u32) -> u32 {
    if out.is_null() { return 0 }
    let s = shared();
    let mut q = s.queue.lock().unwrap();
    let batch = batch_size().min(capacity as usize);

    while q.queue.len() < batch && !q.finished {
        q = s.available.wait(q).unwrap();
    }
    let count = q.queue.len().min(batch);
    for i in 0..count {
        let ev = q.queue.pop_front().unwrap();
        if ev.value != q.previous + 1 { q.errors += 1; }
        q.previous = ev.value;
        q.processed += 1;
        q.latency.push(ev.at.elapsed().as_nanos() as u64);
        out.add(i).write(ev.value);
    }
    s.space.notify_one();
    count as u32
}

#[no_mangle]
pub extern "C" fn bwl_batches_finish(sum: u32) {
    let s = shared();
    let q = s.queue.lock().unwrap();
    let elapsed = s.start.lock().unwrap().unwrap().elapsed();
    let mut latencies = q.latency.clone();
    latencies.sort_unstable();
    let n = latencies.len();
    let expected = (EVENTS as u64 * (EVENTS as u64 + 1) / 2) as u32;
    println!(
        "BEND_BATCH_RESULT batch={} pause_us={} events={} checksum={} expected={} invalid={} remaining={} high_water={} elapsed_ms={:.3} throughput_per_s={:.0} p50_ns={} p99_ns={}",
        batch_size(), env::var("BWL_PAUSE_US").unwrap_or_default(),
        q.processed, sum, expected, q.errors, q.queue.len(), q.high_water,
        elapsed.as_secs_f64() * 1000.0,
        q.processed as f64 / elapsed.as_secs_f64(),
        latencies.get(n / 2).copied().unwrap_or(0),
        latencies.get(n * 99 / 100).copied().unwrap_or(0)
    );

    if q.processed != EVENTS || sum != expected || q.errors != 0 || !q.queue.is_empty() {
        std::process::abort();
    }
}
