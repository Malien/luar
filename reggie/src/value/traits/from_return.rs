use crate::{Machine, sized_value::SizedValue, FromMultiReturnPart, LuaValue, TryFromReturn};

pub trait FromReturn<'a>
where
    Self: Sized,
{
    fn from_machine_state(machine: &'a Machine, return_count: u16) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Strict<T>(pub T);

// pub struct TryStrict<T>(pub T);

// pub struct CannotCollectReturn;

impl<'a> FromReturn<'a> for () {
    fn from_machine_state(_: &'a Machine, _: u16) -> Self {
        ()
    }
}

impl<'a, T> FromReturn<'a> for Strict<T>
where
    T: FromReturn<'a> + SizedValue,
{
    fn from_machine_state(machine: &'a Machine, return_count: u16) -> Self {
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

impl<'a, T> FromReturn<'a> for Result<T, T::Error> where T: TryFromReturn<'a> {
    fn from_machine_state(machine: &'a Machine, return_count: u16) -> Self {
        T::try_from_machine_state(machine, return_count)
    }
}

impl<'a, T> FromReturn<'a> for T 
where 
    T: FromMultiReturnPart<'a, 0> 
{
    fn from_machine_state(machine: &'a Machine, return_count: u16) -> Self {
        if return_count == 0 {
            T::from_absent_value(machine)
        } else {
            T::from_multi_return(machine)
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
        impl<'a, $($generic,)+> FromReturn<'a> for ($($generic,)+)
        where
            $($generic: FromMultiReturnPart<'a, $pos>),+
        {
            fn from_machine_state(machine: &'a Machine, return_count: u16) -> Self {
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
    3 => (D, [A+B+C+D-]),
}

impl<'a> FromReturn<'a> for &'a [LuaValue] {
    fn from_machine_state(machine: &'a Machine, return_count: u16) -> Self {
        &machine.argument_registers.d[..(return_count as usize)]
    }
}
