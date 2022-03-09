macro_rules! fib_bench {
    ($name: ident) => {
        pub mod $name {
            use criterion::Criterion;
            use luar::{
                lang::{Eval, EvalContextExt, GlobalContext, LuaValue},
                syn::string_parser,
            };
            use mlua;

            static BENCH_FILE: &'static str =
                include_str!(concat!("../lua_benches/", stringify!($name), ".lua"));

            pub fn bench(c: &mut Criterion) {
                let mut group = c.benchmark_group(concat!(stringify!($name), ".lua"));

                group.bench_function("ast interpretation", |b| {
                    let module = string_parser::module(BENCH_FILE).unwrap();
                    let mut context = GlobalContext::new();
                    context.set("N", LuaValue::number(20));

                    b.iter(|| {
                        module.eval(&mut context).unwrap();
                    });
                });

                group.bench_function("lua 5.4", |b| {
                    let lua = mlua::Lua::new();
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
