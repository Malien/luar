use crate::{Machine, LuaValue};

pub trait FromMultiReturnPart<'a, const N: u16> {
    fn from_multi_return(machine: &'a Machine) -> Self;
    fn from_absent_value(machine: &'a Machine) -> Self;
}

impl<'a, const N: u16> FromMultiReturnPart<'a, N> for LuaValue {
    fn from_multi_return(machine: &'a Machine) -> Self {
        machine.argument_registers.d[N as usize].clone()
    }

    fn from_absent_value(_: &'a Machine) -> Self {
        LuaValue::NIL
    }
}

impl<'a, const N: u16> FromMultiReturnPart<'a, N> for &'a LuaValue {
    fn from_multi_return(machine: &'a Machine) -> Self {
        &machine.argument_registers.d[N as usize]
    }

    fn from_absent_value(machine: &'a Machine) -> Self {
        machine.global_values.global_nil()
    }
}

impl<'a, const N: u16> FromMultiReturnPart<'a, N> for bool {
    fn from_multi_return(machine: &'a Machine) -> Self {
        machine.argument_registers.d[N as usize].is_truthy()
    }

    fn from_absent_value(_: &'a Machine) -> Self {
        false
    }
}
