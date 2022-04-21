use crate::{machine::ArgumentRegisters, LuaValue};

pub trait FromArgs {
    fn from_args(argument_registers: &mut ArgumentRegisters, arg_count: u16) -> Self;
}

impl FromArgs for () {
    fn from_args(_: &mut ArgumentRegisters, _: u16) -> Self {
        ()
    }
}

impl FromArgs for (LuaValue,) {
    fn from_args(argument_registers: &mut ArgumentRegisters, argument_count: u16) -> Self {
        if argument_count > 0 {
            (argument_registers.d[0].clone(),)
        } else {
            (LuaValue::Nil,)
        }
    }
}

impl FromArgs for (LuaValue, LuaValue) {
    fn from_args(argument_registers: &mut ArgumentRegisters, argument_count: u16) -> Self {
        match argument_count {
            0 => (LuaValue::Nil, LuaValue::Nil),
            1 => (argument_registers.d[0].clone(), LuaValue::Nil),
            _ => (
                argument_registers.d[0].clone(),
                argument_registers.d[1].clone(),
            ),
        }
    }
}
