use criterion::{criterion_group, criterion_main};

mod benchmarks;
use benchmarks::*;

criterion_group!(benches, fib_rec::bench, fib_tailrec::bench, fib_loop::bench);
criterion_main!(benches);
