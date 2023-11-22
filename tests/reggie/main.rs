#[macro_use]
extern crate quickcheck_macros;

use luar_lex::{NumberLiteral, StringLiteral, Token};
use quickcheck::TestResult;
use reggie::{eval_module, eval_str, lua_format, value::Strict, LuaError, LuaValue, Machine};

mod assignment;
mod boolean_ops;
mod comparison;
mod function;
mod local_decl;
mod table;
mod table_constructor;
mod unary_op;
mod while_loop;

pub fn eq_with_nan(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        true
    } else if a.is_infinite() && b.is_infinite() {
        a.is_sign_negative() == b.is_sign_negative()
    } else {
        a == b
    }
}

#[test]
fn eval_empty() -> Result<(), LuaError> {
    let mut machine = Machine::new();
    eval_str("", &mut machine)
}

#[test]
fn eval_nil() -> Result<(), LuaError> {
    let mut machine = Machine::new();
    assert_eq!(
        eval_str::<Strict<LuaValue>>("return nil", &mut machine)?.0,
        LuaValue::Nil
    );
    Ok(())
}

#[quickcheck]
fn eval_number_literal(num: f64) -> Result<TestResult, LuaError> {
    use luar_syn::unspanned_lua_token_parser;

    if !num.is_finite() {
        return Ok(TestResult::discard());
    }
    let module =
        unspanned_lua_token_parser::module([Token::Return, Token::Number(NumberLiteral(num))])?;
    let mut machine = Machine::new();
    let res = eval_module::<Strict<LuaValue>>(&module, &mut machine)?
        .0
        .number_as_f64()
        .unwrap();
    assert!(eq_with_nan(res, num));
    Ok(TestResult::passed())
}

#[quickcheck]
fn eval_string_literal(str: String) -> Result<(), LuaError> {
    use luar_syn::unspanned_lua_token_parser;

    let expected_value = LuaValue::string(&str);
    let module =
        unspanned_lua_token_parser::module([Token::Return, Token::String(StringLiteral(str))])?;
    let mut context = Machine::new();
    assert_eq!(
        eval_module::<Strict<&LuaValue>>(&module, &mut context)?.0,
        &expected_value
    );
    Ok(())
}

#[quickcheck]
fn value_is_equal_to_itself(value: LuaValue) -> Result<TestResult, LuaError> {
    if let LuaValue::Float(num) = value {
        if num.is_nan() {
            // NaN does not equal itself
            return Ok(TestResult::discard());
        }
    }

    let mut machine = Machine::new();
    machine.global_values.set("value", value);
    let res = eval_str("return value == value", &mut machine)?;
    assert_eq!(LuaValue::true_value(), res);
    Ok(TestResult::passed())
}

#[quickcheck]
fn different_values_do_not_equal_themselves(lhs: LuaValue, rhs: LuaValue) -> Result<(), LuaError> {
    let expected = LuaValue::from_bool(lhs == rhs);
    let mut machine = Machine::new();
    machine.global_values.set("lhs", lhs);
    machine.global_values.set("rhs", rhs);
    let res = eval_str("return lhs == rhs", &mut machine)?;
    assert_eq!(expected, res);
    Ok(())
}

#[quickcheck]
fn not_equals_is_the_negation_of_equality(lhs: LuaValue, rhs: LuaValue) -> Result<(), LuaError> {
    let mut machine = Machine::new();
    machine.global_values.set("lhs", lhs);
    machine.global_values.set("rhs", rhs);
    let res = eval_str("return (not (lhs ~= rhs)) == (lhs == rhs)", &mut machine)?;
    assert_eq!(LuaValue::true_value(), res);
    Ok(())
}

#[quickcheck]
fn concat(lhs: LuaValue, rhs: LuaValue) {
    let mut machine = Machine::new();
    machine.global_values.set("lhs", lhs.clone());
    machine.global_values.set("rhs", rhs.clone());

    let res = eval_str::<LuaValue>("return lhs .. rhs", &mut machine);
    if let (Some(lhs), Some(rhs)) = (lhs.coerce_to_string(), rhs.coerce_to_string()) {
        let res = res.unwrap();
        assert!(res.total_eq(&LuaValue::String(lua_format!("{lhs}{rhs}"))));
    } else {
        assert!(res.is_err());
    }
}
