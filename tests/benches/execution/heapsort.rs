use criterion::Criterion;
use luar_syn::lua_parser;

static BENCH_FILE: &'static str = include_str!(concat!("../lua_benches/heapsort.lua"));

fn random_ast_vm_tbl(size: usize) -> ast_vm::lang::TableValue {
    let mut table = ast_vm::lang::TableValue::new();
    for i in 1..=size {
        table.set(
            ast_vm::lang::LuaKey::number(i),
            ast_vm::lang::LuaValue::number(random()),
        );
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

fn random_reggie_tbl(size: usize) -> reggie::TableValue {
    let mut table = reggie::TableValue::new();
    for _ in 0..size {
        table.push(reggie::LuaValue::Float(random()))
    }
    table
}

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("heapsort.lua");
    // Calling srand is always safe
    unsafe { libc::srand(42) };

    group.bench_function("ast interpretation", |b| {
        let module = lua_parser::module(BENCH_FILE).unwrap();

        b.iter_batched(
            || {
                let mut context = ast_vm::stdlib::std_context();
                context.set(
                    "TABLE",
                    ast_vm::lang::LuaValue::table(random_ast_vm_tbl(10_000)),
                );
                context.set("COUNT", ast_vm::lang::LuaValue::number(10_000i32));
                context
            },
            |mut context| {
                ast_vm::eval_module(&module, &mut context).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("reggie baseline", |b| {
        let module = lua_parser::module(BENCH_FILE).unwrap();

        b.iter_batched(
            || {
                let mut machine = reggie::Machine::with_stdlib();
                let compiled_module =
                    reggie::compiler::compile_module(&module, &mut machine.global_values);
                let block = machine.code_blocks.add_module(compiled_module);
                machine.global_values.set(
                    "TABLE",
                    reggie::LuaValue::Table(reggie::TableRef::from(random_reggie_tbl(10_000))),
                );
                machine
                    .global_values
                    .set("COUNT", reggie::LuaValue::Int(10_000));
                (block, machine)
            },
            |(block, mut machine)| {
                reggie::call_block::<()>(block, &mut machine).unwrap();
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
