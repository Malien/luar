use criterion::criterion_group;

mod heapsort;

macro_rules! fib_bench {
    ($name: ident) => {
        pub mod $name {
            static BENCH_FILE: &'static str =
                include_str!(concat!("../lua_benches/", stringify!($name), ".lua"));

            pub fn bench(c: &mut ::criterion::Criterion) {
                let mut group = c.benchmark_group(concat!(stringify!($name), ".lua"));

                group.bench_function("ast interpretation", |b| {
                    let module = ::luar::syn::lua_parser::module(BENCH_FILE).unwrap();
                    let mut context = ::luar::lang::GlobalContext::new();
                    ::luar::lang::GlobalContext::set(
                        &mut context,
                        "N",
                        ::luar::lang::LuaValue::number(20),
                    );

                    b.iter(|| {
                        ::luar::ast_vm::eval_module(&module, &mut context).unwrap();
                    });
                });

                group.bench_function("lua 5.4", |b| {
                    let lua = ::mlua::Lua::new();
                    lua.globals().set("N", 20).unwrap();
                    let routine = lua.load(BENCH_FILE).into_function().unwrap();

                    b.iter(|| routine.call::<(), ()>(()));
                });
            }
        }
    };
}

fib_bench!(fib_rec);
fib_bench!(fib_tailrec);
fib_bench!(fib_loop);

criterion_group!(
    benches,
    fib_rec::bench,
    fib_tailrec::bench,
    fib_loop::bench,
    heapsort::bench
);
