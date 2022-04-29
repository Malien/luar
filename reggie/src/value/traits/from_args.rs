use crate::{machine::ArgumentRegisters, FromArgsPart};

pub trait FromArgs<'a> {
    fn from_args(argument_registers: &'a ArgumentRegisters, arg_count: u16) -> Self;
}

impl<'a> FromArgs<'a> for () {
    fn from_args(_: &'a ArgumentRegisters, _: u16) -> Self {
        ()
    }
}

macro_rules! match_arm {
    ($argument_registers: expr, $type: ident-) => {
        $type::from_absent_argument()
    };
    ($argument_registers: expr, $type: ident+) => {
        $type::from_argument($argument_registers)
    };
}

macro_rules! impl_from_args_tuple {
    ($($pos: expr => ($generic: ident, [$($gen: ident $present: tt)+])),+$(,)?) => {
        impl<'a, $($generic,)+> FromArgs<'a> for ($($generic,)+)
        where
            $($generic: FromArgsPart<'a, $pos>),+
        {
            fn from_args(argument_registers: &'a ArgumentRegisters, arg_count: u16) -> Self {
                match arg_count {
                    $($pos => ($(match_arm!{ argument_registers, $gen $present },)+),)+
                    _ => ($($generic::from_argument(argument_registers),)+)
                }
            }
        }
    };
}

impl_from_args_tuple! { 
    0 => (A, [A-])
}
impl_from_args_tuple! { 
    0 => (A, [A-B-]),
    1 => (B, [A+B-]),
}
impl_from_args_tuple! { 
    0 => (A, [A-B-C-]),
    1 => (B, [A+B-C-]),
    2 => (C, [A+B+C-]),
}
impl_from_args_tuple! { 
    0 => (A, [A-B-C-D-]),
    1 => (B, [A+B-C-D-]),
    2 => (C, [A+B+C-D-]),
    3 => (D, [A+B+C+D-]),
}

// impl<'a, T> FromArgs<'a> for (T,) 
// where
//     T: FromArgsPart<'a, 0>
// {
//     fn from_args(argument_registers: &'a ArgumentRegisters, arg_count: u16) -> Self {
//         // if argumn
//     }
// }

// impl FromArgs for (LuaValue,) {
//     fn from_args(argument_registers: &mut ArgumentRegisters, argument_count: u16) -> Self {
//         if argument_count > 0 {
//             (argument_registers.d[0].clone(),)
//         } else {
//             (LuaValue::Nil,)
//         }
//     }
// }

// impl FromArgs for (LuaValue, LuaValue) {
//     fn from_args(argument_registers: &mut ArgumentRegisters, argument_count: u16) -> Self {
//         match argument_count {
//             0 => (LuaValue::Nil, LuaValue::Nil),
//             1 => (argument_registers.d[0].clone(), LuaValue::Nil),
//             _ => (
//                 argument_registers.d[0].clone(),
//                 argument_registers.d[1].clone(),
//             ),
//         }
//     }
// }
