use crate::{LuaValue, Machine};

pub struct Strict<T>(pub T);

// pub struct TryStrict<T>(pub T);

pub struct CannotCollectReturn;

pub trait FromLuaReturn<'a>
where
    Self: Sized,
{
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self;
}

pub trait TryFromLuaReturn<'a>
where
    Self: Sized,
{
    type Error;

    fn try_from_machine_state(machine: &'a Machine, return_count: usize) -> Result<Self, Self::Error>;
}


trait FromMultiReturnPart<'a, const N: usize> {
    fn from_multi_return(machine: &'a Machine) -> Self;
    fn from_absent_value(machine: &'a Machine) -> Self;
}

pub trait TryFromMultiReturnPart<'a, const N: usize> where Self: Sized {
    type Error;

    fn try_from_multi_return(machine: &'a Machine) -> Result<Self, Self::Error>;
    fn try_from_absent_value(machine: &'a Machine) -> Result<Self, Self::Error>;
}

trait SizedLuaReturn {
    const COUNT: usize;
}

impl<'a> FromLuaReturn<'a> for () {
    fn from_machine_state(_: &'a Machine, _: usize) -> Self {
        ()
    }
}

macro_rules! count {
    () => { 0usize };
    ($x:tt $($xs:tt)*) => (1usize + count!($($xs) *))
}

macro_rules! impl_return_size_tuple {
    ($($gen: ident) +) => {
        impl<$($gen),+> SizedLuaReturn for ($($gen,)+) {
            const COUNT: usize = count!($($gen) +);
        }
    };
}

impl SizedLuaReturn for () {
    const COUNT: usize = 0;
}

impl SizedLuaReturn for LuaValue {
    const COUNT: usize = 1;
}

impl SizedLuaReturn for bool {
    const COUNT: usize = 1;
}

impl_return_size_tuple! { A }
impl_return_size_tuple! { A B }
impl_return_size_tuple! { A B C }
impl_return_size_tuple! { A B C D }

impl<'a, T> FromLuaReturn<'a> for Strict<T>
where
    T: FromLuaReturn<'a> + SizedLuaReturn,
{
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        assert_eq!(
            return_count, 
            T::COUNT, 
            "Cannot convert {} returned values into Strict<{}>, which requires exactly {} values to be returned", 
            return_count, 
            std::any::type_name::<T>(), 
            T::COUNT
        );
        Strict(T::from_machine_state(machine, return_count))
    }
}

// impl<'a, T> FromLuaReturn<'a> for TryStrict<T>
// where
//     T: TryFromLuaReturn<'a> + SizedLuaReturn,
//     T::Error: std::fmt::Debug
// {
//     fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
//         assert_eq!(
//             return_count, 
//             T::COUNT, 
//             "Cannot convert {} returned values into Strict<{}>, which requires exactly {} values to be returned", 
//             return_count, 
//             std::any::type_name::<T>(), 
//             T::COUNT
//         );
//         TryStrict(T::try_from_machine_state(machine, return_count).unwrap())
//     }
// }


impl<'a, T> FromLuaReturn<'a> for Result<T, T::Error> where T: TryFromLuaReturn<'a> {
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        T::try_from_machine_state(machine, return_count)
    }
}

impl<'a, T> FromLuaReturn<'a> for T 
where 
    T: FromMultiReturnPart<'a, 0> 
{
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        if return_count == 0 {
            T::from_multi_return(machine)
        } else {
            T::from_absent_value(machine)
        }
    }
}

macro_rules! match_arm {
    ($machine: expr, $type: ident-) => {
        $type::from_absent_value($machine)
    };
    ($machine: expr, $type: ident+) => {
        $type::from_multi_return($machine)
    };
}

macro_rules! impl_from_lua_return_tuple {
    ($($pos: expr => ($generic: ident, [$($gen: ident $present: tt)+])),+$(,)?) => {
        impl<'a, $($generic,)+> FromLuaReturn<'a> for ($($generic,)+)
        where
            $($generic: FromMultiReturnPart<'a, $pos>),+
        {
            fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
                match return_count {
                    $($pos => ($(match_arm!{ machine, $gen $present },)+),)+
                    _ => ($($generic::from_multi_return(machine),)+)
                }
            }
        }
    };
}

impl_from_lua_return_tuple! { 
    0 => (A, [A-])
}
impl_from_lua_return_tuple! { 
    0 => (A, [A-B-]),
    1 => (B, [A+B-]),
}
impl_from_lua_return_tuple! { 
    0 => (A, [A-B-C-]),
    1 => (B, [A+B-C-]),
    2 => (C, [A+B+C-]),
}
impl_from_lua_return_tuple! { 
    0 => (A, [A-B-C-D-]),
    1 => (B, [A+B-C-D-]),
    2 => (C, [A+B+C-D-]),
    4 => (D, [A+B+C+D-]),
}

// impl<'a, T, U> FromLuaReturn<'a> for (T,U)
// where
//     T: FromMultiReturnPart<'a, 0>,
//     U: FromMultiReturnPart<'a, 1>
// {
//     fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
//         match return_count {
//             0 => (T::from_absent_value(machine), U::from_absent_value(machine)),
//             1 => (T::from_multi_return(machine), U::from_absent_value(machine)),
//             _ => (T::from_multi_return(machine), U::from_multi_return(machine)),
//         }
//     }
// }


impl<'a> FromLuaReturn<'a> for &'a [LuaValue] {
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        &machine.argument_registers.d[..return_count]
    }
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

impl<'a, T> TryFromLuaReturn<'a> for T 
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

impl<'a, T, U> TryFromLuaReturn<'a> for (T, U)
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
