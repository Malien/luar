use crate::{
    machine::ArgumentRegisters,
    signature::{ArgumentType, FunctionSignatureList},
    EvalError, LuaValue,
};

pub trait ReturnRepresentable {
    fn returns() -> FunctionSignatureList;
    fn to_lua_return(self, argument_registers: &mut ArgumentRegisters) -> Result<(), EvalError>;
    fn return_count() -> u16;
}

impl ReturnRepresentable for () {
    fn returns() -> FunctionSignatureList {
        FunctionSignatureList::Finite(vec![])
    }
    fn to_lua_return(self, _: &mut ArgumentRegisters) -> Result<(), EvalError> {
        Ok(())
    }

    fn return_count() -> u16 {
        0
    }
}

impl ReturnRepresentable for LuaValue {
    fn returns() -> FunctionSignatureList {
        FunctionSignatureList::Finite(vec![ArgumentType::Dynamic])
    }
    fn to_lua_return(self, argument_registers: &mut ArgumentRegisters) -> Result<(), EvalError> {
        argument_registers.d[0] = self;
        Ok(())
    }
    fn return_count() -> u16 {
        1
    }
}

impl ReturnRepresentable for (LuaValue, LuaValue) {
    fn returns() -> FunctionSignatureList {
        FunctionSignatureList::Finite(vec![ArgumentType::Dynamic, ArgumentType::Dynamic])
    }
    fn to_lua_return(self, argument_registers: &mut ArgumentRegisters) -> Result<(), EvalError> {
        argument_registers.d[0] = self.0;
        argument_registers.d[1] = self.1;
        Ok(())
    }
    fn return_count() -> u16 {
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
    fn to_lua_return(self, argument_registers: &mut ArgumentRegisters) -> Result<(), EvalError> {
        self.and_then(|value| value.to_lua_return(argument_registers))
    }
    fn return_count() -> u16 {
        T::return_count()
    }
}
