#[cfg(test)]
#[cfg(feature = "quickcheck")]
#[macro_use]
extern crate quickcheck_macros;

pub(crate) mod compiler;
pub(crate) mod eq_with_nan;
pub(crate) mod ids;
pub(crate) mod machine;
pub(crate) mod meta;
pub(crate) mod ops;
pub(crate) mod runtime;
pub mod stdlib;
pub mod value;
pub(crate) mod keyed_vec;

use compiler::CompiledModule;
use ids::BlockID;
pub use machine::Machine;
pub use machine::GlobalValues;
use machine::ProgramCounter;
use machine::StackFrame;
use meta::ReturnCount;
use runtime::eval_loop;
pub use value::*;

pub type LuaError = luar_error::LuaError<LuaValue>;
pub type EvalError = luar_error::EvalError<LuaValue>;
pub type TypeError = luar_error::TypeError<LuaValue>;
pub type ArithmeticError = luar_error::ArithmeticError<LuaValue>;

use value::FromReturn;

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
    call_module(compiled_module, machine)
}

pub fn call_module<'a, T: FromReturn<'a>>(
    module: CompiledModule,
    machine: &'a mut Machine,
) -> Result<T, EvalError> {
    let top_level_block = machine.code_blocks.add_module(module);
    call_block(machine, top_level_block)
}

pub fn call_block<'a, T: FromReturn<'a>>(
    machine: &'a mut Machine,
    block_id: BlockID,
) -> Result<T, EvalError> {
    let block = &machine.code_blocks[block_id];
    let return_count = block.meta.return_count;
    let stack_frame = StackFrame::new(
        &block.meta,
        ProgramCounter {
            block: BlockID(0),
            position: 0,
        },
    );
    machine.stack.push(stack_frame);
    machine.program_counter = ProgramCounter {
        block: block_id,
        position: 0,
    };
    eval_loop(machine)?;
    let return_count = match return_count {
        ReturnCount::Constant(count) => count,
        _ => machine.value_count,
    };
    Ok(T::from_machine_state(machine, return_count))
}

