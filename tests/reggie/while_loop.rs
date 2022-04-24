use luar_syn::lua_parser;
use reggie::{eval_module, LuaError, LuaValue, Machine, Strict};

#[quickcheck]
fn while_loop_executes_until_condition_is_true(times: u8) -> Result<(), LuaError> {
    let module = lua_parser::module(
        "count_executed = 0
        while i ~= 0 do
            count_executed = count_executed + 1
            i = i - 1
        end
        return i, count_executed",
    )?;
    let mut machine = Machine::new();
    machine.global_values.set("i", LuaValue::Int(times as i32));
    let Strict((i, count_executed)) =
        eval_module::<Strict<(&LuaValue, &LuaValue)>>(&module, &mut machine)?;
    assert_eq!(i, &LuaValue::Int(0));
    assert_eq!(count_executed, &LuaValue::Int(times as i32));
    Ok(())
}
