//! A bounded, multi-threaded producer and a Bend-driven native consumer.
//! This is an experimental benchmark, not a production compositor queue.

use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex, OnceLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const EVENTS: u64 = 100_000;
const CAPACITY: usize = 8192;
const BATCH: usize = 256;

#[derive(Clone, Copy)]
struct Event {
    sequence: u64,
    enqueued: Instant,
}

struct Queue {
    events: VecDeque<Event>,
    finished: bool,
    consumed: u64,
    invalid: u64,
    high_water: usize,
    latencies_ns: Vec<u64>,
    previous: u64,
}

struct Shared {
    state: Mutex<Queue>,
    available: Condvar,
    space: Condvar,
    producer: Mutex<Option<JoinHandle<()>>>,
    start: Mutex<Option<Instant>>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

fn shared() -> &'static Shared {
    SHARED.get_or_init(|| Shared {
        state: Mutex::new(Queue {
            events: VecDeque::with_capacity(CAPACITY),
            finished: false,
            consumed: 0,
            invalid: 0,
            high_water: 0,
            latencies_ns: Vec::with_capacity(EVENTS as usize),
            previous: 0,
        }),
        available: Condvar::new(),
        space: Condvar::new(),
        producer: Mutex::new(None),
        start: Mutex::new(None),
    })
}

#[no_mangle]
pub extern "C" fn bwl_bench_start() {
    let s = shared();
    *s.start.lock().unwrap() = Some(Instant::now());
    let handle = thread::spawn(|| {
        let s = shared();
        for sequence in 1..=EVENTS {
            let mut state = s.state.lock().unwrap();
            while state.events.len() == CAPACITY {
                state = s.space.wait(state).unwrap();
            }
            state.events.push_back(Event {
                sequence,
                enqueued: Instant::now(),
            });
            state.high_water = state.high_water.max(state.events.len());
            s.available.notify_one();
        }

        let mut state = s.state.lock().unwrap();
        state.finished = true;
        s.available.notify_all();
    });
    *s.producer.lock().unwrap() = Some(handle);
}

#[no_mangle]
pub extern "C" fn bwl_bench_consume() {
    let s = shared();
    let mut state = s.state.lock().unwrap();

    while state.events.is_empty() && !state.finished {
        let (next, _) = s.available
            .wait_timeout(state, Duration::from_millis(25))
            .unwrap();
        state = next;
    }

    // The native layer returns at most BATCH events per Bend effect.
    // We account for each event here. The Bend program schedules each batch.
    let now = Instant::now();
    for _ in 0..BATCH {
        let Some(event) = state.events.pop_front() else { break };
        if event.sequence != state.previous + 1 {
            state.invalid += 1;
        }
        state.previous = event.sequence;
        state.consumed += 1;
        state.latencies_ns.push(
            now.duration_since(event.enqueued).as_nanos().min(u64::MAX as u128) as u64
        );
    }
    s.space.notify_one();
}

#[no_mangle]
pub extern "C" fn bwl_bench_finish() {
    let s = shared();
    // The bounded loop must have had enough iterations to drain all events.
    // Join can otherwise block if the producer is stalled by a full queue.
    let (consumed, invalid, high_water, latencies) = {
        let state = s.state.lock().unwrap();
        (state.consumed, state.invalid, state.high_water, state.latencies_ns.clone())
    };

    let duration = s.start.lock().unwrap().unwrap().elapsed();
    let mut latencies = latencies;
    latencies.sort_unstable();
    let n = latencies.len();
    let p50 = latencies.get(n / 2).copied().unwrap_or(0);
    let p99 = latencies.get(n.saturating_mul(99) / 100).copied().unwrap_or(0);
    let sec = duration.as_secs_f64();

    println!(
        "BEND_CONSUMER_RESULT events={} invalid={} remaining={} high_water={} elapsed_ms={:.3} throughput_per_s={:.0} p50_ns={} p99_ns={}",
        consumed, invalid, EVENTS.saturating_sub(consumed), high_water,
        sec * 1000.0, consumed as f64 / sec, p50, p99
    );

    // A failing integrity check must propagate through the GitHub Action.
    if consumed != EVENTS || invalid != 0 {
        eprintln!("FAIL: incomplete or out-of-order Bend-driven consumption");
        std::process::abort();
    }
    s.producer.lock().unwrap().take().unwrap().join().unwrap();
}
