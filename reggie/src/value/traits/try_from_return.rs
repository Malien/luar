use crate::{Machine, TryFromMultiReturnPart};

pub trait TryFromReturn<'a>
where
    Self: Sized,
{
    type Error;

    fn try_from_machine_state(machine: &'a Machine, return_count: usize) -> Result<Self, Self::Error>;
}

impl<'a, T> TryFromReturn<'a> for T 
where 
    T: TryFromMultiReturnPart<'a, 0> 
{
    type Error = T::Error;
    
    fn try_from_machine_state(machine: &'a Machine, return_count: usize) -> Result<Self, Self::Error> {
        if return_count == 0 {
            T::try_from_multi_return(machine)
        } else {
            T::try_from_absent_value(machine)
        }
    }
}

#[derive(Debug)]
pub enum Tuple2ConversionError<A, B> {
    First(A),
    Second(B)
}

impl<A> Tuple2ConversionError<A, A> {
    pub fn common_error(self) -> A {
        match self {
            Self::First(a) => a,
            Self::Second(a) => a,
        }
    }
}

impl<'a, T, U> TryFromReturn<'a> for (T, U)
where 
    T: TryFromMultiReturnPart<'a, 0>,
    U: TryFromMultiReturnPart<'a, 1> 
{
    type Error = Tuple2ConversionError<T::Error, U::Error>;
    
    fn try_from_machine_state(machine: &'a Machine, return_count: usize) -> Result<Self, Self::Error> {
        let first = match return_count {
            0 => T::try_from_absent_value(machine),
            _ => T::try_from_multi_return(machine),
        };
        let first = match first {
            Err(err) => return Err(Tuple2ConversionError::First(err)),
            Ok(first) => first,
        };
        let second = match return_count {
            0..=1 => U::try_from_absent_value(machine),
            _ => U::try_from_multi_return(machine),
        };
        let second = match second {
            Err(err) => return Err(Tuple2ConversionError::Second(err)),
            Ok(second) => second,
        };

        Ok((first, second))
    }
}
