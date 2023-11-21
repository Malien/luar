use criterion::criterion_group;

mod heapsort;
mod fib;
mod string_packing;

use fib::*;

criterion_group!(
    benches,
    fib_rec::bench,
    fib_tailrec::bench,
    fib_loop::bench,
    heapsort::bench,
    string_packing::bench
);
