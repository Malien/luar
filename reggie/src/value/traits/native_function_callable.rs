use std::marker::PhantomData;

use crate::{FFIFunc, FromArgs, Machine, ReturnRepresentable};

pub trait NativeFunctionCallable {
    fn call(&self, machine: &mut Machine);
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
    fn call(&self, machine: &mut Machine) {
        let args = Args::from_args(machine, machine.value_count);
        let res = self.func.call(args);
        // TODO: cause error from within FFI calls to propagate inside of the VM
        res.to_lua_return(machine).unwrap();
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
