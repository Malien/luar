use criterion::{Bencher, BenchmarkId, Criterion};
use luar_syn::lua_parser;
use rand::{distributions::Alphanumeric, Rng};

static BENCH_FILE: &str = include_str!("../lua_benches/string_packing.lua");

fn random_ascii_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .take(len)
        .map(char::from)
        .collect()
}

fn bench_reggie(b: &mut Bencher, i: &usize) {
    let module = lua_parser::module(BENCH_FILE).unwrap();

    b.iter_batched(
        || {
            let mut machine = reggie::Machine::with_stdlib();
            let compiled_module =
                reggie::compiler::compile_module(&module, &mut machine.global_values);
            let block = machine.code_blocks.add_module(compiled_module);
            let input = reggie::LuaValue::String(random_ascii_string(*i));
            machine.global_values.set("INPUT", input);
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
    let routine = lua.load(BENCH_FILE).into_function().unwrap();

    b.iter_batched(
        || {
            lua.globals()
                .set("INPUT", random_ascii_string(*i))
                .unwrap()
        },
        |()| routine.call::<(), ()>(()),
        criterion::BatchSize::SmallInput,
    )
}

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_packing.lua");

    let input_lengths = [1, 4, 16, 64, 256, 1024, 4096];

    for i in input_lengths.into_iter() {
        group.bench_with_input(BenchmarkId::new("Reggie baseline", i), &i, bench_reggie);
        group.bench_with_input(BenchmarkId::new("Lua 5.4", i), &i, bench_lua);
    }
}
