use crate::{machine::ArgumentRegisters, LuaValue};

pub trait FromArgsPart<'a, const N: u16> {
    fn from_argument(argument_registers: &'a ArgumentRegisters) -> Self;
    fn from_absent_argument() -> Self;
}

impl<'a, const N: u16> FromArgsPart<'a, N> for LuaValue {
    fn from_argument(argument_registers: &'a ArgumentRegisters) -> Self {
        argument_registers.d[N as usize].clone()
    }

    fn from_absent_argument() -> Self {
        LuaValue::Nil
    }
}

impl<'a, const N: u16> FromArgsPart<'a, N> for &'a LuaValue {
    fn from_argument(argument_registers: &'a ArgumentRegisters) -> Self {
        &argument_registers.d[N as usize]
    }

    fn from_absent_argument() -> Self {
        &LuaValue::Nil
    }
}
