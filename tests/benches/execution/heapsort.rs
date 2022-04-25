use ast_vm::{
    lang::{LuaKey, LuaValue, TableValue},
    stdlib::std_context,
};
use criterion::Criterion;
use luar_syn::lua_parser;

static BENCH_FILE: &'static str = include_str!(concat!("../lua_benches/heapsort.lua"));

fn random_luar_tbl(size: usize) -> TableValue {
    let mut table = TableValue::new();
    for i in 1..=size {
        table.set(LuaKey::number(i), LuaValue::number(random()));
    }
    table
}

fn random() -> f64 {
    (unsafe { libc::rand() } as f64) / (libc::INT_MAX as f64)
}

fn random_mlua_tbl(lua: &mlua::Lua, size: usize) -> mlua::Table {
    lua.create_table_from(
        std::iter::repeat_with(random)
            .enumerate()
            .skip(1)
            .take(size),
    )
    .unwrap()
}

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("heapsort.lua");

    group.bench_function("ast interpretation", |b| {
        let module = lua_parser::module(BENCH_FILE).unwrap();

        b.iter_batched(
            || {
                let mut context = std_context();
                context.set("TABLE", LuaValue::table(random_luar_tbl(10_000)));
                context.set("COUNT", LuaValue::number(10_000i32));
                context
            },
            |mut context| {
                ast_vm::eval_module(&module, &mut context).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("lua 5.4", |b| {
        let lua = mlua::Lua::new();
        lua.globals().set("COUNT", 10_000i32).unwrap();
        let routine = lua.load(BENCH_FILE).into_function().unwrap();

        b.iter_batched(
            || lua.globals().set("TABLE", random_mlua_tbl(&lua, 10_000)),
            |_| routine.call::<(), ()>(()),
            criterion::BatchSize::SmallInput,
        );
    });
}
