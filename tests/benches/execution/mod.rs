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
                    let module = ::luar_syn::lua_parser::module(BENCH_FILE).unwrap();
                    let mut context = ::ast_vm::lang::GlobalContext::new();
                    ::ast_vm::lang::GlobalContext::set(
                        &mut context,
                        "N",
                        ::ast_vm::lang::LuaValue::number(20i32),
                    );

                    b.iter(|| {
                        ::ast_vm::ast_vm::eval_module(&module, &mut context).unwrap();
                    });
                });

                group.bench_function("reggie baseline", |b| {
                    let module = ::luar_syn::lua_parser::module(BENCH_FILE).unwrap();
                    let mut machine = ::reggie::Machine::new();
                    let compiled_module = ::reggie::compiler::compile_module(&module, &mut machine.global_values);
                    let top_level_block = machine.code_blocks.add_module(compiled_module);
                    machine.global_values.set("N", ::reggie::LuaValue::Int(20));

                    b.iter(|| {
                        ::reggie::call_block::<()>(top_level_block, &mut machine).unwrap();
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
