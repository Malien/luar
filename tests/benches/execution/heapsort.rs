use criterion::{Bencher, BenchmarkId, Criterion};
use luar_syn::lua_parser;

static BENCH_FILE: &'static str = include_str!("../lua_benches/heapsort.lua");

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

fn bench_ast(b: &mut Bencher, i: &usize) {
    let module = lua_parser::module(BENCH_FILE).unwrap();

    b.iter_batched(
        || {
            let mut context = ast_vm::stdlib::std_context();
            context.set(
                "TABLE",
                ast_vm::lang::LuaValue::table(random_ast_vm_tbl(*i)),
            );
            context.set("COUNT", ast_vm::lang::LuaValue::number(*i));
            context
        },
        |mut context| {
            ast_vm::eval_module(&module, &mut context).unwrap();
        },
        criterion::BatchSize::SmallInput,
    );
}

fn bench_ast_opt(b: &mut Bencher, i: &usize) {
    use ast_vm::lang::LuaValue;
    let module = lua_parser::module(BENCH_FILE).unwrap();

    b.iter_batched(
        || {
            let mut context = ast_vm::stdlib::std_context();
            let module = ast_vm::opt::compile_module(module.clone(), &mut context.globals);
            context.set("TABLE", LuaValue::table(random_ast_vm_tbl(*i)));
            context.set("COUNT", LuaValue::number(*i));
            (module, context)
        },
        |(module, mut context)| {
            ast_vm::opt::eval_module(&module, &mut context).unwrap();
        },
        criterion::BatchSize::SmallInput,
    );
}

fn bench_reggie(b: &mut Bencher, i: &usize) {
    let module = lua_parser::module(BENCH_FILE).unwrap();

    b.iter_batched(
        || {
            let mut machine = reggie::Machine::with_stdlib();
            let compiled_module =
                reggie::compiler::compile_module(&module, &mut machine.global_values);
            let block = machine.code_blocks.add_module(compiled_module);
            machine.global_values.set(
                "TABLE",
                reggie::LuaValue::Table(reggie::TableRef::from(random_reggie_tbl(*i))),
            );
            machine
                .global_values
                .set("COUNT", reggie::LuaValue::Int(*i as i32));
            (block, machine)
        },
        |(block, mut machine)| {
            reggie::call_block::<()>(block, &mut machine).unwrap();
        },
        criterion::BatchSize::SmallInput,
    );
}

fn bench_lua(b: &mut Bencher, i: &usize) {
    let lua = mlua::Lua::new();
    lua.globals().set("COUNT", *i).unwrap();
    let routine = lua.load(BENCH_FILE).into_function().unwrap();

    b.iter_batched(
        || lua.globals().set("TABLE", random_mlua_tbl(&lua, *i)),
        |_| routine.call::<(), ()>(()),
        criterion::BatchSize::SmallInput,
    );
}

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("heapsort.lua");
    // Calling srand is always safe
    unsafe { libc::srand(42) };

    let inputs = [64, 128, 256, 512, 1024, 2048, 4096];
    let len = inputs.len();
    let sample_count = 100 / len;

    for (idx, i) in inputs.into_iter().enumerate() {
        group.sample_size((len - idx) * sample_count);

        group.bench_with_input(BenchmarkId::new("AST interpretation", i), &i, bench_ast);
        group.bench_with_input(BenchmarkId::new("AST optimized", i), &i, bench_ast_opt);
        group.bench_with_input(BenchmarkId::new("Reggie baseline", i), &i, bench_reggie);
        group.bench_with_input(BenchmarkId::new("Lua 5.4", i), &i, bench_lua);
    }
}
