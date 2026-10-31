//! Genuine event values cross the Bend native-effect boundary.
//! The Bend program calculates the checksum and passes it back to Rust.

use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex, OnceLock},
    thread,
    time::Instant,
};

const EVENTS: u32 = 20_000;
const CAPACITY: usize = 1024;

struct Event {
    value: u32,
    timestamp: Instant,
}

struct State {
    queue: VecDeque<Event>,
    sent: bool,
    consumed: usize,
    maximum_depth: usize,
    latencies_ns: Vec<u64>,
}

struct Shared {
    state: Mutex<State>,
    available: Condvar,
    space: Condvar,
    start: Mutex<Option<Instant>>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

fn shared() -> &'static Shared {
    SHARED.get_or_init(|| Shared {
        state: Mutex::new(State {
            queue: VecDeque::with_capacity(CAPACITY),
            sent: false,
            consumed: 0,
            maximum_depth: 0,
            latencies_ns: Vec::with_capacity(EVENTS as usize),
        }),
        available: Condvar::new(),
        space: Condvar::new(),
        start: Mutex::new(None),
    })
}

#[no_mangle]
pub extern "C" fn bwl_events_start() {
    let s = shared();
    *s.start.lock().unwrap() = Some(Instant::now());

    thread::spawn(|| {
        let s = shared();
        for value in 1..=EVENTS {
            let mut state = s.state.lock().unwrap();
            while state.queue.len() == CAPACITY {
                state = s.space.wait(state).unwrap();
            }
            state.queue.push_back(Event {
                value,
                timestamp: Instant::now(),
            });
            state.maximum_depth = state.maximum_depth.max(state.queue.len());
            s.available.notify_one();
        }

        let mut state = s.state.lock().unwrap();
        state.sent = true;
        s.available.notify_all();
    });
}

#[no_mangle]
pub extern "C" fn bwl_events_next() -> u32 {
    let s = shared();
    let mut state = s.state.lock().unwrap();

    while state.queue.is_empty() && !state.sent {
        state = s.available.wait(state).unwrap();
    }

    let event = state.queue.pop_front().expect("Bend requested more events than generated");
    let age = event.timestamp.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    state.latencies_ns.push(age);
    state.consumed += 1;
    s.space.notify_one();
    event.value
}

#[no_mangle]
pub extern "C" fn bwl_events_finish(checksum: u32) {
    let s = shared();
    let state = s.state.lock().unwrap();
    let elapsed = s.start.lock().unwrap().unwrap().elapsed();
    let expected = EVENTS as u64 * (EVENTS as u64 + 1) / 2;

    let mut latencies = state.latencies_ns.clone();
    latencies.sort_unstable();
    let n = latencies.len();
    let p50 = latencies.get(n / 2).copied().unwrap_or(0);
    let p99 = latencies.get(n * 99 / 100).copied().unwrap_or(0);

    println!(
        "BEND_VALUE_RESULT events={} checksum={} expected={} queue_remaining={} high_water={} elapsed_ms={:.3} throughput_per_s={:.0} p50_ns={} p99_ns={}",
        state.consumed,
        checksum,
        expected,
        state.queue.len(),
        state.maximum_depth,
        elapsed.as_secs_f64() * 1000.0,
        state.consumed as f64 / elapsed.as_secs_f64(),
        p50,
        p99
    );

    if checksum as u64 != expected
        || state.consumed != EVENTS as usize
        || !state.queue.is_empty()
    {
        eprintln!("FAIL: event transfer or Bend-side accumulation");
        std::process::abort();
    }
}
