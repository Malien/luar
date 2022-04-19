use crate::{
    signature::{ArgumentType, FunctionSignatureList},
    EvalError, Machine, LuaValue,
};

pub trait ReturnRepresentable {
    fn returns() -> FunctionSignatureList;
    fn to_lua_return(self, machine: &mut Machine) -> Result<(), EvalError>;
    fn return_count() -> usize;
}

impl ReturnRepresentable for () {
    fn returns() -> FunctionSignatureList {
        FunctionSignatureList::Finite(vec![])
    }
    fn to_lua_return(self, _: &mut Machine) -> Result<(), EvalError> {
        Ok(())
    }

    fn return_count() -> usize {
        0
    }
}

impl ReturnRepresentable for LuaValue {
    fn returns() -> FunctionSignatureList {
        FunctionSignatureList::Finite(vec![ArgumentType::Dynamic])
    }
    fn to_lua_return(self, machine: &mut Machine) -> Result<(), EvalError> {
        machine.argument_registers.d[0] = self;
        Ok(())
    }
    fn return_count() -> usize {
        1
    }
}

impl ReturnRepresentable for (LuaValue, LuaValue) {
    fn returns() -> FunctionSignatureList {
        FunctionSignatureList::Finite(vec![ArgumentType::Dynamic, ArgumentType::Dynamic])
    }
    fn to_lua_return(self, machine: &mut Machine) -> Result<(), EvalError> {
        machine.argument_registers.d[0] = self.0;
        machine.argument_registers.d[1] = self.1;
        Ok(())
    }
    fn return_count() -> usize {
        2
    }
}

impl<T> ReturnRepresentable for Result<T, EvalError>
where
    T: ReturnRepresentable,
{
    fn returns() -> FunctionSignatureList {
        T::returns()
    }
    fn to_lua_return(self, machine: &mut Machine) -> Result<(), EvalError> {
        self.and_then(|value| value.to_lua_return(machine))
    }
    fn return_count() -> usize {
        T::return_count()
    }
}
