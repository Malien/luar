#[macro_export]
macro_rules! basic_expr_tests {
    ($engine: ty, $context: expr) => {
        mod basic_expr {
            use ::luar::error::LuaError;
            use ::luar::lang::{Engine, ReturnValue};
            use ::luar::lex::{NumberLiteral, StringLiteral, Token};
            use ::luar::syn::{lua_parser, unspanned_lua_token_parser};
            use ::quickcheck::TestResult;
            use ::quickcheck_macros::quickcheck;

            #[test]
            fn eval_nil() -> Result<(), LuaError> {
                let module = lua_parser::module("return nil")?;
                let mut context = $context;
                assert_eq!(
                    <$engine>::eval_module(&module, &mut context)?,
                    ReturnValue::Nil
                );
                Ok(())
            }

            #[quickcheck]
            fn eval_number_literal(num: f64) -> Result<TestResult, LuaError> {
                if !num.is_finite() {
                    return Ok(TestResult::discard());
                }
                let module = unspanned_lua_token_parser::module([
                    Token::Return,
                    Token::Number(NumberLiteral(num)),
                ])?;
                let mut context = $context;
                assert!(<$engine>::eval_module(&module, &mut context)?
                    .total_eq(&ReturnValue::number(num)));
                Ok(TestResult::passed())
            }

            #[quickcheck]
            fn eval_string_literal(str: String) -> Result<(), LuaError> {
                let module = unspanned_lua_token_parser::module([
                    Token::Return,
                    Token::String(StringLiteral(str.clone())),
                ])?;
                let mut context = $context;
                assert_eq!(
                    <$engine>::eval_module(&module, &mut context)?,
                    ReturnValue::String(str)
                );
                Ok(())
            }
        }
    };
}
