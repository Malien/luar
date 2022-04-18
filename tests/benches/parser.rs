use criterion::{criterion_group, Criterion};

macro_rules! parser_bench {
    ($name: expr, $group: expr) => {{
        let group = &mut ($group);
        static BENCH_FILE: &'static str = include_str!(concat!("./lua_benches/", $name, ".lua"));

        group.bench_function(concat!($name, ".lua"), |b| {
            b.iter(|| {
                criterion::black_box(::luar::syn::lua_parser::module(BENCH_FILE).unwrap());
            })
        });
    }};
}

fn parser_benches_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    parser_bench!("fib_rec", group);
    parser_bench!("fib_tailrec", group);
    parser_bench!("fib_loop", group);
}

criterion_group!(benches, parser_benches_group);
