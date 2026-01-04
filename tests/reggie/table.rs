use luar_lex::Ident;
use luar_syn::lua_parser;
use quickcheck::TestResult;
use reggie::{
    LuaValue, Machine, TableRef, TableValue, TypeError, assert_type_error, eval_module,
    match_type_error,
};

#[quickcheck]
fn accessing_non_table_property_is_an_error(value: LuaValue, property: Ident) -> TestResult {
    if value.is_table() {
        return TestResult::discard();
    }

    let module = lua_parser::module(&format!("return value.{}", property)).unwrap();
    let mut machine = Machine::new();
    machine.global_values.set("value", value);
    let res = eval_module::<()>(&module, &mut machine);
    assert_type_error!(TypeError::CannotAccessProperty { .. }, res);
    TestResult::passed()
}

#[quickcheck]
fn accessing_non_table_member_is_an_error(value: LuaValue) -> TestResult {
    if value.is_table() {
        return TestResult::discard();
    }

    let module = lua_parser::module("return value[42]").unwrap();
    let mut machine = Machine::new();
    machine.global_values.set("value", value);
    let res = eval_module::<()>(&module, &mut machine);

    match_type_error! {
        let TypeError::CannotAccessMember { member, .. } = res;
        assert_eq!(member, &LuaValue::int(42));
        TestResult::passed()
    }
}

#[quickcheck]
fn assigning_to_a_non_table_property_is_an_error(value: LuaValue, property: Ident) -> TestResult {
    if value.is_table() {
        return TestResult::discard();
    }

    let mut machine = Machine::new();
    machine.global_values.set("value", value);
    let module = lua_parser::module(&format!("value.{} = 69", property)).unwrap();
    let res = eval_module::<()>(&module, &mut machine);
    assert_type_error!(TypeError::CannotAssignProperty { .. }, res);
    TestResult::passed()
}

#[quickcheck]
fn assigning_to_a_non_table_member_is_an_error(value: LuaValue) -> TestResult {
    if value.is_table() {
        return TestResult::discard();
    }

    let mut machine = Machine::new();
    machine.global_values.set("value", value);
    let module = lua_parser::module("value[42] = 69").unwrap();
    let res = eval_module::<()>(&module, &mut machine);

    match_type_error! {
        let TypeError::CannotAssignMember { member, .. } = res;
        assert_eq!(member, &LuaValue::int(42));
        TestResult::passed()
    }
}

#[test]
fn assigning_to_nil_member_is_an_error() {
    let mut machine = Machine::new();
    let module = lua_parser::module("local tbl = {} tbl[nil] = 42").unwrap();
    let res = eval_module::<()>(&module, &mut machine);

    match_type_error! {
        let TypeError::NilAssign(value) = res;
        assert_eq!(value, &LuaValue::int(42));
    };
}

#[test]
fn assigning_to_a_nan_member_is_an_error() {
    let mut machine = Machine::new();
    let module = lua_parser::module("local tbl, nan = {}, 0/0 tbl[nan] = 42").unwrap();
    let res = eval_module::<()>(&module, &mut machine);

    match_type_error! {
        let TypeError::NaNAssign(value) = res;
        assert_eq!(value, &LuaValue::int(42));
    };
}
