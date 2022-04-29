use luar_syn::lua_parser;
use quickcheck::TestResult;
use reggie::{LuaValue, LuaError, Machine, eval_module};

#[quickcheck]
fn eval_and_truthy(lhs: LuaValue, rhs: LuaValue) -> Result<TestResult, LuaError> {
    if lhs.is_falsy() {
        return Ok(TestResult::discard());
    }
    let module = lua_parser::module("return lhs and rhs")?;
    let mut machine = Machine::new();
    machine.global_values.set("lhs", lhs);
    machine.global_values.set("rhs", rhs.clone());
    let res: LuaValue = eval_module(&module, &mut machine)?;
    assert!(res.total_eq(&rhs));
    Ok(TestResult::passed())
}

#[quickcheck]
fn eval_and_falsy(rhs: LuaValue) -> Result<(), LuaError> {
    let module = lua_parser::module("return nil and rhs")?;
    let mut machine = Machine::new();
    machine.global_values.set("rhs", rhs.clone());
    let res: LuaValue = eval_module(&module, &mut machine)?;
    assert_eq!(res, LuaValue::Nil);
    Ok(())
}

#[quickcheck]
fn eval_or_truthy(lhs: LuaValue, rhs: LuaValue) -> Result<TestResult, LuaError> {
    if lhs.is_falsy() {
        return Ok(TestResult::discard());
    }
    let module = lua_parser::module("return lhs or rhs")?;
    let mut machine = Machine::new();
    machine.global_values.set("lhs", lhs.clone());
    machine.global_values.set("rhs", rhs);
    let res: LuaValue = eval_module(&module, &mut machine)?;
    assert!(res.total_eq(&lhs));
    Ok(TestResult::passed())
}

#[quickcheck]
fn eval_or_falsy(rhs: LuaValue) -> Result<(), LuaError> {
    let module = lua_parser::module("return nil or rhs")?;
    let mut machine = Machine::new();
    machine.global_values.set("rhs", rhs.clone());
    let res: LuaValue = eval_module(&module, &mut machine)?;
    assert!(res.total_eq(&rhs));
    Ok(())
}
