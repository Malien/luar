use crate::{Machine, LuaValue};

pub trait FromMultiReturnPart<'a, const N: usize> {
    fn from_multi_return(machine: &'a Machine) -> Self;
    fn from_absent_value(machine: &'a Machine) -> Self;
}

impl<'a, const N: usize> FromMultiReturnPart<'a, N> for LuaValue {
    fn from_multi_return(machine: &'a Machine) -> Self {
        machine.argument_registers.d[N].clone()
    }

    fn from_absent_value(_: &'a Machine) -> Self {
        LuaValue::Nil
    }
}

impl<'a, const N: usize> FromMultiReturnPart<'a, N> for &'a LuaValue {
    fn from_multi_return(machine: &'a Machine) -> Self {
        &machine.argument_registers.d[N]
    }

    fn from_absent_value(machine: &'a Machine) -> Self {
        machine.global_values.global_nil()
    }
}

impl<'a, const N: usize> FromMultiReturnPart<'a, N> for bool {
    fn from_multi_return(machine: &'a Machine) -> Self {
        machine.argument_registers.d[N].is_truthy()
    }

    fn from_absent_value(_: &'a Machine) -> Self {
        false
    }
}
