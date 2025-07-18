use crate::Machine;

pub trait TryFromMultiReturnPart<'a, const N: u16>
where
    Self: Sized,
{
    type Error;

    fn try_from_multi_return(machine: &'a Machine) -> Result<Self, Self::Error>;
    fn try_from_absent_value(machine: &'a Machine) -> Result<Self, Self::Error>;
}

#[derive(Debug)]
pub struct NotAString;

impl<'a, const N: u16> TryFromMultiReturnPart<'a, N> for &'a str {
    type Error = NotAString;

    fn try_from_multi_return(machine: &'a Machine) -> Result<Self, Self::Error> {
        if let Some(str) = machine.argument_registers.d[N as usize].as_str() {
            Ok(str.as_ref())
        } else {
            Err(NotAString)
        }
    }

    fn try_from_absent_value(_: &'a Machine) -> Result<Self, Self::Error> {
        Err(NotAString)
    }
}
