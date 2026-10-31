use std::{collections::VecDeque, sync::{Mutex, Condvar, OnceLock}, time::{Duration, Instant}};
#[repr(C)]
#[derive(Clone,Copy,Default)]
pub struct Event { pub sequence: u64, pub created_ns: u64 }
struct Queue { items: VecDeque<Event>, origin: Instant, capacity: usize, dropped: u64 }
struct Shared { queue: Mutex<Queue>, ready: Condvar }
static STATE: OnceLock<Shared> = OnceLock::new();
fn state() -> &'static Shared { STATE.get_or_init(|| Shared {queue:Mutex::new(Queue{items:VecDeque::with_capacity(8192),origin:Instant::now(),capacity:8192,dropped:0}),ready:Condvar::new()}) }
fn elapsed_ns(q:&Queue)->u64 {q.origin.elapsed().as_nanos().min(u64::MAX as u128) as u64}
#[no_mangle] pub extern "C" fn bwl_now_ns()->u64 { let q=state().queue.lock().unwrap();elapsed_ns(&q) }
#[no_mangle] pub extern "C" fn bwl_enqueue(sequence:u64)->i32 {
 let s=state(); let mut q=s.queue.lock().unwrap();
 if q.items.len()==q.capacity {q.dropped+=1;return 0}
 let created_ns=elapsed_ns(&q);q.items.push_back(Event{sequence,created_ns});s.ready.notify_one();1
}
#[no_mangle] pub unsafe extern "C" fn bwl_drain(out:*mut Event, max:usize)->usize {
 if out.is_null() || max==0{return 0}
 let mut q=state().queue.lock().unwrap();let n=max.min(q.items.len());
 for i in 0..n {out.add(i).write(q.items.pop_front().unwrap())} n
}
#[no_mangle] pub extern "C" fn bwl_wait_ms(timeout_ms:u32)->i32 {
 let s=state();let q=s.queue.lock().unwrap();
 if !q.items.is_empty(){return 1}
 let (q,_)=s.ready.wait_timeout(q,Duration::from_millis(timeout_ms as u64)).unwrap();
 (!q.items.is_empty()) as i32
}
#[no_mangle] pub extern "C" fn bwl_dropped()->u64 {state().queue.lock().unwrap().dropped}
