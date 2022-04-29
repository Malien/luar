use std::marker::PhantomData;

use crate::{machine::ArgumentRegisters, EvalError, FFIFunc, FromArgs, ReturnRepresentable};

pub trait NativeFunctionCallable {
    fn call(
        &self,
        argument_registers: &mut ArgumentRegisters,
        value_count: u16,
    ) -> Result<(), EvalError>;
    fn return_count(&self) -> u16;
}

pub struct NativeFunctionWrapper<F, Args> {
    func: F,
    _args: PhantomData<Args>,
}

impl<'a, F, Args> NativeFunctionCallable for NativeFunctionWrapper<F, Args>
where
    F: FFIFunc<Args>,
    Args: FromArgs<'a>,
{
    fn call(
        &self,
        argument_registers: &mut ArgumentRegisters,
        value_count: u16,
    ) -> Result<(), EvalError> {
        // SAFETY: Look, I'm dumb. I can't figure out for the life of me the lifetimes here.
        //         The idea is, that result of Args::from_args is valid for the lifetime
        //         that &mut ArgumentRegisters is valid for (which is the duration of this call).
        //         Since I'm using the args only for duration of the call (by passing it into
        //         self.func.call(args)), there is no harm being done. FromArgs::from_args returns
        //         the value that is valid for the same lifetime as &mut argument_registers.
        //         &mut argument_registers lives for the duration of the call, meaning it is safe
        //         to use args for the same duration. I mean... There is probably UB somewhere,
        //         but I'm fed up with this!
        let args = Args::from_args(
            unsafe { &mut *(argument_registers as *mut ArgumentRegisters) },
            value_count,
        );
        let res = self.func.call(args);
        res.to_lua_return(argument_registers)
    }

    fn return_count(&self) -> u16 {
        F::Output::return_count()
    }
}

impl<'a, F, Args> NativeFunctionWrapper<F, Args>
where
    F: FFIFunc<Args> + 'static,
    Args: FromArgs<'a> + 'static,
{
    pub fn new(func: F) -> Self {
        Self {
            func,
            _args: PhantomData,
        }
    }
}
