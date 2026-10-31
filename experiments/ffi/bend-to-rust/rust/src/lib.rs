use std::sync::atomic::{AtomicU32, Ordering};
static CALLS: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
pub extern "C" fn bwl_rust_probe_ping() {
    let n = CALLS.fetch_add(1, Ordering::SeqCst) + 1;
    println!("RUST_EFFECT_CALLED:{n}");
}
