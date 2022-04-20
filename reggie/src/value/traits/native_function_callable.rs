use std::marker::PhantomData;

use crate::{machine::ArgumentRegisters, FFIFunc, FromArgs, ReturnRepresentable, EvalError};

pub trait NativeFunctionCallable {
    fn call(&self, argument_registers: &mut ArgumentRegisters, value_count: u32) -> Result<(), EvalError>;
    fn return_count(&self) -> usize;
}

pub struct NativeFunctionWrapper<F, Args> {
    func: F,
    _args: PhantomData<Args>,
}

impl<F, Args> NativeFunctionCallable for NativeFunctionWrapper<F, Args>
where
    F: FFIFunc<Args>,
    Args: FromArgs,
{
    fn call(&self, argument_registers: &mut ArgumentRegisters, value_count: u32) -> Result<(), EvalError>{
        let args = Args::from_args(argument_registers, value_count);
        let res = self.func.call(args);
        res.to_lua_return(argument_registers)
    }

    fn return_count(&self) -> usize {
        F::Output::return_count()
    }
}

impl<F, Args> NativeFunctionWrapper<F, Args>
where
    F: FFIFunc<Args> + 'static,
    Args: FromArgs + 'static,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _args: PhantomData,
        }
    }
}
