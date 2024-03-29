#[cfg(test)]
#[cfg(feature = "quickcheck")]
#[macro_use]
extern crate quickcheck_macros;

pub mod compiler;
pub(crate) mod eq_with_nan;
pub mod global_values;
pub(crate) mod ids;
pub(crate) mod machine;
pub(crate) mod meta;
pub(crate) mod ops;
pub(crate) mod optimizer;
pub(crate) mod runtime;
pub mod stdlib;
pub mod value;
pub(crate) mod call_stack;

use compiler::CompiledModule;
pub use global_values::GlobalValues;
use ids::BlockID;
pub use machine::Machine;
use meta::ReturnCount;
pub use value::*;

pub mod error;
pub use error::*;

pub fn eval_str<'a, T: FromReturn<'a>>(
    module_str: &str,
    machine: &'a mut Machine,
) -> Result<T, LuaError> {
    let module = luar_syn::lua_parser::module(module_str)?;
    eval_module(&module, machine).map_err(LuaError::from)
}

pub fn eval_module<'a, T: FromReturn<'a>>(
    module: &luar_syn::Module,
    machine: &'a mut Machine,
) -> Result<T, EvalError> {
    let compiled_module = compiler::compile_module(&module, &mut machine.global_values);
    eval_compiled_module(compiled_module, machine)
}

pub fn eval_compiled_module<'a, T: FromReturn<'a>>(
    module: CompiledModule,
    machine: &'a mut Machine,
) -> Result<T, EvalError> {
    let top_level_block = machine.code_blocks.add_module(module);
    call_block(top_level_block, machine)
}

pub fn call_block<'a, T: FromReturn<'a>>(
    block_id: BlockID,
    machine: &'a mut Machine,
) -> Result<T, EvalError> {
    let block = &machine.code_blocks[block_id];
    trace_execution!(
        "Calling block from top-level {block_id:?} {}",
        block
            .meta
            .debug_name
            .as_ref()
            .map(String::as_str)
            .unwrap_or_default()
    );
    let return_count = block.meta.return_count;

    if let Err(err) = runtime::execute(machine, block_id) {
        if !machine.stack.is_empty() {
            let last_fn = machine.program_counter.block;
            let last_fn = &machine.code_blocks[last_fn];
            machine.stack.clear(&last_fn.meta, &machine.code_blocks);
        }
        return Err(err);
    }
    let return_count = match return_count {
        ReturnCount::Constant(count) => count,
        _ => machine.value_count,
    };
    Ok(T::from_machine_state(machine, return_count))
}

#[macro_export]
#[cfg(feature = "trace-execution")]
macro_rules! trace_execution {
    ($($fmt:expr),*) => {
        println!($($fmt,)*);
    };
}

#[macro_export]
#[cfg(not(feature = "trace-execution"))]
macro_rules! trace_execution {
    ($($fmt:expr),*) => {};
}

#[test]
fn lua_value_is_still_16_bytes() {
    assert_eq!(std::mem::size_of::<LuaValue>(), 16);
}
