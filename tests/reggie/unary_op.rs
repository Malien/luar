use reggie::{eval_str, LuaError, LuaValue, Machine, Strict};

#[quickcheck]
fn not(value: LuaValue) -> Result<(), LuaError> {
    let mut machine = Machine::new();
    let is_truthy = value.is_truthy();
    machine.global_values.set("value", value);
    let Strict(res) = eval_str::<Strict<LuaValue>>("return not value", &mut machine)?;

    if is_truthy {
        assert_eq!(res, LuaValue::Nil);
    } else {
        assert_eq!(res, LuaValue::Int(1));
    }
    Ok(())
}
