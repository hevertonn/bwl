//! ABI reference tests. These call our C fixture, NOT Bend functions.
//! The Bend integration is separately exercised by scripts/validate.sh.
use std::ffi::c_void;
use std::hint::black_box;
use std::time::Instant;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pair { x: u32, y: u32 }

unsafe extern "C" {
    fn fixture_add(a: u32, b: u32) -> u32;
    fn fixture_swap(p: Pair) -> Pair;
    fn fixture_checksum(p: *const u8, n: usize) -> u64;
    fn fixture_call(cb: extern "C" fn(u32, *mut c_void) -> u32, arg: u32, ctx: *mut c_void) -> u32;
}

extern "C" fn rust_callback(x: u32, _: *mut c_void) -> u32 { x.wrapping_add(10) }

fn bench<F: FnMut() -> u32>(label: &str, mut f: F) {
    const N: u64 = 2_000_000;
    let start = Instant::now();
    let mut sink = 0u32;
    for _ in 0..N { sink = sink.wrapping_add(black_box(f())); }
    black_box(sink);
    println!("BENCH {label}: {:.2} ns/call ({} iterations; shared CI runner, illustrative only)", start.elapsed().as_nanos() as f64 / N as f64, N);
}

fn main() {
    unsafe {
        assert_eq!(fixture_add(19, 23), 42, "scalar Rust -> C");
        assert_eq!(fixture_swap(Pair { x: 5, y: 8 }), Pair { x: 8, y: 5 }, "repr(C) struct Rust -> C");
        let bytes = [0u8, 1, 2, 3, 254, 255];
        assert_eq!(fixture_checksum(bytes.as_ptr(), bytes.len()), 515, "borrowed buffer Rust -> C");
        assert_eq!(fixture_call(rust_callback, 32, std::ptr::null_mut()), 42, "C -> Rust callback");
        println!("PASS: Rust/C baseline scalar, repr(C) struct, borrowed bytes, callback");
        bench("rust-to-c scalar baseline", || fixture_add(20, 22));
        bench("c-to-rust callback baseline", || fixture_call(rust_callback, 32, std::ptr::null_mut()));
    }
}
