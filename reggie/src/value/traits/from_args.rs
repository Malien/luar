use crate::{Machine, LuaValue};

pub trait FromArgs {
    fn from_args(machine: &mut Machine, arg_count: u32) -> Self;
}

impl FromArgs for () {
    fn from_args(_: &mut Machine, _: u32) -> Self {
        ()
    }
}

impl FromArgs for (LuaValue,) {
    fn from_args(machine: &mut Machine, argument_count: u32) -> Self {
        if argument_count > 0 {
            (machine.argument_registers.d[0].clone(),)
        } else {
            (LuaValue::Nil,)
        }
    }
}

impl FromArgs for (LuaValue, LuaValue) {
    fn from_args(machine: &mut Machine, argument_count: u32) -> Self {
        match argument_count {
            0 => (LuaValue::Nil, LuaValue::Nil),
            1 => (machine.argument_registers.d[0].clone(), LuaValue::Nil),
            _ => (
                machine.argument_registers.d[0].clone(),
                machine.argument_registers.d[1].clone(),
            ),
        }
    }
}