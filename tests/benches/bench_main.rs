use criterion::criterion_main;

mod execution;
mod parser;

criterion_main!(execution::benches, parser::benches);
