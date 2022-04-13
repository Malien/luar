pub(crate) mod compiler;
pub(crate) mod ids;
pub(crate) mod machine;
pub(crate) mod meta;
pub(crate) mod ops;
pub(crate) mod runtime;

pub use machine::Machine;

use crate::{
    error::LuaError,
    lang::{EvalError, LuaValue, ReturnValue},
    syn,
    util::NonEmptyVec,
};

pub fn eval_str(module_str: &str, machine: &mut Machine) -> Result<ReturnValue, LuaError> {
    let module = syn::lua_parser::module(module_str)?;
    let res = eval_module(&module, machine)?;
    Ok(res)
}

pub fn eval_module(module: &syn::Module, machine: &mut Machine) -> Result<ReturnValue, EvalError> {
    let compiled_module = compiler::compile_module(&module, &mut machine.global_values);
    let returns = runtime::call_module(compiled_module, machine)?;
    Ok(to_ffi_return_value(returns))
}

fn to_ffi_return_value(values: &[LuaValue]) -> ReturnValue {
    if values.len() == 0 {
        return ReturnValue::Nil;
    }
    if values.len() == 1 {
        return ReturnValue::from(values.first().unwrap().clone());
    }
    let cloned_values = Vec::from_iter(values.into_iter().cloned());
    // SAFETY: cloned_values always contains at least two elements since two pervious checks failed.
    let return_values = unsafe { NonEmptyVec::new_unchecked(cloned_values) };
    return ReturnValue::MultiValue(return_values);
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use crate::{
        error::LuaError,
        lang::{LuaValue, ReturnValue},
        lex::{NumberLiteral, StringLiteral, Token},
        reggie::{compiler::compile_module, eval_module, eval_str, machine::Machine, runtime::call_module},
        syn::{lua_parser, unspanned_lua_token_parser},
        test_util::Finite,
    };

    #[test]
    fn eval_empty() -> Result<(), LuaError> {
        let module = lua_parser::module("")?;
        let mut machine = Machine::new();
        let compiled_module = compile_module(&module, &mut machine.global_values);
        let res = call_module(compiled_module, &mut machine)?;
        assert_eq!(res, []);
        Ok(())
    }

    #[test]
    fn eval_nil() -> Result<(), LuaError> {
        let module = lua_parser::module("return nil")?;
        let mut machine = Machine::new();
        let compiled_module = compile_module(&module, &mut machine.global_values);
        let res = call_module(compiled_module, &mut machine)?;
        assert_eq!(res, [LuaValue::Nil]);
        Ok(())
    }

    #[quickcheck]
    fn eval_number_literal(Finite(num): Finite<f64>) -> Result<(), LuaError> {
        let module =
            unspanned_lua_token_parser::module([Token::Return, Token::Number(NumberLiteral(num))])?;
        let mut machine = Machine::new();
        assert!(eval_module(&module, &mut machine)?.total_eq(&ReturnValue::number(num)));
        Ok(())
    }

    #[quickcheck]
    fn eval_string_literal(str: String) -> Result<(), LuaError> {
        let module = unspanned_lua_token_parser::module([
            Token::Return,
            Token::String(StringLiteral(str.clone())),
        ])?;
        let mut machine = Machine::new();
        assert_eq!(
            eval_module(&module, &mut machine)?,
            ReturnValue::String(str)
        );
        Ok(())
    }

    #[quickcheck]
    fn value_is_equal_to_itself(value: LuaValue) -> Result<TestResult, LuaError> {
        if let LuaValue::Number(num) = value {
            if num.as_f64().is_nan() {
                // NaN does not equal itself
                return Ok(TestResult::discard());
            }
        }

        let mut machine = Machine::new();
        machine.global_values.set("value", value);
        let res = eval_str("return value == value", &mut machine)?;
        assert_eq!(LuaValue::true_value(), res.assert_single());
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn different_values_do_not_equal_themselves(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<(), LuaError> {
        let expected = LuaValue::from_bool(lhs == rhs);
        let module = lua_parser::module("return lhs == rhs")?;
        let mut machine = Machine::new();
        machine.global_values.set("lhs", lhs);
        machine.global_values.set("rhs", rhs);
        let res = eval_module(&module, &mut machine)?;
        assert_eq!(expected, res.assert_single());
        Ok(())
    }

    #[quickcheck]
    fn not_equals_is_the_negation_of_equality(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return (not (lhs ~= rhs)) == (lhs == rhs)")?;
        let mut machine = Machine::new();
        machine.global_values.set("lhs", lhs);
        machine.global_values.set("rhs", rhs);
        let res = eval_module(&module, &mut machine)?;
        assert_eq!(LuaValue::true_value(), res.assert_single());
        Ok(())
    }

}
