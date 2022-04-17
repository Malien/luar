use crate::{LuaValue, Machine};

pub struct Strict<T>(pub T);

pub struct CannotCollectReturn;

pub trait FromLuaReturn<'a>
where
    Self: Sized,
{
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self;
}

impl<'a> FromLuaReturn<'a> for () {
    fn from_machine_state(_: &'a Machine, _: usize) -> Self {
        ()
    }
}

impl<'a> FromLuaReturn<'a> for Strict<()> {
    fn from_machine_state(_: &'a Machine, return_count: usize) -> Self {
        assert!(
            return_count == 0,
            "Expected to return 0 values (converted to ()), but got {}",
            return_count
        );
        Strict(())
    }
}

impl<'a> FromLuaReturn<'a> for Result<(), CannotCollectReturn> {
    fn from_machine_state(_: &'a Machine, return_count: usize) -> Self {
        if return_count == 0 {
            Ok(())
        } else {
            Err(CannotCollectReturn)
        }
    }
}

impl<'a> FromLuaReturn<'a> for LuaValue {
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        if return_count > 0 {
            machine.argument_registers.d[0].clone()
        } else {
            LuaValue::Nil
        }
    }
}

impl<'a> FromLuaReturn<'a> for Result<LuaValue, CannotCollectReturn> {
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        if return_count == 1 {
            Ok(machine.argument_registers.d[0].clone())
        } else {
            Err(CannotCollectReturn)
        }
    }
}

impl<'a> FromLuaReturn<'a> for Strict<LuaValue> {
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        assert!(
            return_count == 1,
            "Cannot convert return of 0 values into a LuaValue"
        );
        Strict(machine.argument_registers.d[0].clone())
    }
}

impl<'a> FromLuaReturn<'a> for &'a LuaValue {
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        if return_count > 0 {
            &machine.argument_registers.d[0]
        } else {
            machine.global_values.global_nil()
        }
    }
}

impl<'a> FromLuaReturn<'a> for Result<&'a LuaValue, CannotCollectReturn> {
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        if return_count == 1 {
            Ok(&machine.argument_registers.d[0])
        } else {
            Err(CannotCollectReturn)
        }
    }
}

impl<'a> FromLuaReturn<'a> for Strict<&'a LuaValue> {
    fn from_machine_state(machine: &'a Machine, return_count: usize) -> Self {
        assert!(
            return_count == 1,
            "Cannot convert return of 0 values into a LuaValue"
        );
        Strict(&machine.argument_registers.d[0])
    }
}
