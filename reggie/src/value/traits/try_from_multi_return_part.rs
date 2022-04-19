use crate::{LuaValue, Machine};


pub trait TryFromMultiReturnPart<'a, const N: usize> where Self: Sized {
    type Error;

    fn try_from_multi_return(machine: &'a Machine) -> Result<Self, Self::Error>;
    fn try_from_absent_value(machine: &'a Machine) -> Result<Self, Self::Error>;
}

#[derive(Debug)]
pub struct NotAString;

impl<'a, const N:usize> TryFromMultiReturnPart<'a, N> for &'a str {
    type Error = NotAString;

    fn try_from_multi_return(machine: &'a Machine) -> Result<Self, Self::Error> {
        match &machine.argument_registers.d[N] {
            LuaValue::String(str) => Ok(str.as_ref()),
            _ => Err(NotAString)
        }
    }

    fn try_from_absent_value(_: &'a Machine) -> Result<Self, Self::Error> {
        Err(NotAString)
    }
}