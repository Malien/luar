macro_rules! fib_bench {
    ($name: ident, $inputs: expr) => {
        pub mod $name {
            static BENCH_FILE: &'static str =
                include_str!(concat!("../lua_benches/", stringify!($name), ".lua"));

            fn bench_ast(b: &mut ::criterion::Bencher, i: &i32) {
                let module = ::luar_syn::lua_parser::module(BENCH_FILE).unwrap();
                let mut context = ::ast_vm::lang::GlobalContext::new();
                ::ast_vm::lang::GlobalContext::set(
                    &mut context,
                    "N",
                    ::ast_vm::lang::LuaValue::number(*i),
                );

                b.iter(|| {
                    ::ast_vm::eval_module(&module, &mut context).unwrap();
                });
            }

            fn bench_reggie(b: &mut ::criterion::Bencher, i: &i32) {
                let module = ::luar_syn::lua_parser::module(BENCH_FILE).unwrap();
                let mut machine = ::reggie::Machine::new();
                let compiled_module =
                    ::reggie::compiler::compile_module(&module, &mut machine.global_values);
                let top_level_block = machine.code_blocks.add_module(compiled_module);
                machine.global_values.set("N", ::reggie::LuaValue::Int(*i));

                b.iter(|| {
                    ::reggie::call_block::<()>(top_level_block, &mut machine).unwrap();
                });
            }

            fn bench_lua(b: &mut ::criterion::Bencher, i: &i32) {
                let lua = ::mlua::Lua::new();
                lua.globals().set("N", *i).unwrap();
                let routine = lua.load(BENCH_FILE).into_function().unwrap();

                b.iter(|| routine.call::<(), ()>(()));
            }

            pub fn bench(c: &mut ::criterion::Criterion) {
                let mut group = c.benchmark_group(concat!(stringify!($name), ".lua"));

                let inputs = $inputs;
                let len = inputs.len();
                let sample_count = 100 / len;

                for (idx, i) in inputs.into_iter().enumerate() {
                    group.sample_size((len - idx) * sample_count);

                    group.bench_with_input(
                        ::criterion::BenchmarkId::new("AST interpretation", i),
                        &i,
                        bench_ast,
                    );

                    group.bench_with_input(
                        ::criterion::BenchmarkId::new("Reggie baseline", i),
                        &i,
                        bench_reggie,
                    );

                    group.bench_with_input(
                        ::criterion::BenchmarkId::new("Lua 5.4", i),
                        &i,
                        bench_lua,
                    );
                }
            }
        }
    };
}

fib_bench!(fib_rec, [0,1,2,4,8,16]);
fib_bench!(fib_tailrec, [0,1,2,4,8,16,32,64,128]);
fib_bench!(fib_loop, [0,1,2,4,8,16,64,128]);
