//! Regression test: an untrusted `matrix.def` header must not take down the
//! process when the declared matrix cannot be allocated.
//!
//! `MatrixConnector::from_reader` used to allocate `vec![0; num_right * num_left]`
//! directly from the header values (each `u16`, so up to ~8.59 GiB). On hosts
//! that cannot satisfy the allocation (e.g. commit-limited Windows processes),
//! `vec![0; len]` aborts the whole process via `handle_alloc_error`.
//!
//! The allocator below deterministically simulates such a host by refusing any
//! single allocation larger than 1 GiB, which makes the test
//! machine-independent: the ~8.59 GiB header-driven request always fails,
//! while all normal (small) allocations pass through unchanged.

use std::alloc::{GlobalAlloc, Layout, System};

/// Refuse single allocations larger than 1 GiB.
const MAX_SINGLE_ALLOCATION: usize = 1024 * 1024 * 1024;

struct Bounded;

unsafe impl GlobalAlloc for Bounded {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() > MAX_SINGLE_ALLOCATION {
            return std::ptr::null_mut();
        }
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: Bounded = Bounded;

use vibrato::SystemDictionaryBuilder;

#[test]
fn untrusted_matrix_def_header_is_graceful_error() {
    let lex = std::fs::read("src/tests/resources/lex.csv").unwrap();
    let char_def = std::fs::read("src/tests/resources/char.def").unwrap();
    let unk = std::fs::read("src/tests/resources/unk.def").unwrap();

    // 11-byte header declaring 65535 x 65535 i16 = ~8.59 GiB.
    let evil_matrix: &[u8] = b"65535 65535";

    let result =
        SystemDictionaryBuilder::from_readers(&lex[..], evil_matrix, &char_def[..], &unk[..]);

    match result {
        Err(_) => { /* graceful: the dictionary builder returned an error. */ }
        Ok(_) => panic!("the declared matrix was silently committed"),
    }
}
