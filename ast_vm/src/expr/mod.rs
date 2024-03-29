use crate::{
    lang::{LocalScope, ReturnValue, ScopeHolder, TableRef, LuaValue},
    EvalError,
};
use luar_lex::{NumberLiteral, StringLiteral};
use luar_syn::Expression;

mod table_constructor;
pub(crate) use table_constructor::eval_tbl_constructor;

pub(crate) mod unary_op;
use unary_op::eval_unary_op_expr;

pub(crate) mod binary_op;
use binary_op::binary_op_eval;

pub(crate) mod fn_call;
pub(crate) use fn_call::eval_fn_call;

use super::eval_var;

pub(crate) fn eval_expr(
    expr: &Expression,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ReturnValue, EvalError> {
    match expr {
        Expression::Nil => Ok(ReturnValue::NIL),
        Expression::Number(NumberLiteral(num)) => Ok(ReturnValue::number(*num)),
        Expression::String(StringLiteral(str)) => Ok(ReturnValue::string(str)),
        Expression::Variable(var) => eval_var(var, scope).map(ReturnValue::from),
        Expression::TableConstructor(tbl) => eval_tbl_constructor(tbl, scope)
            .map(TableRef::from)
            .map(LuaValue::Table)
            .map(ReturnValue::from),
        Expression::FunctionCall(call) => eval_fn_call(call, scope),
        Expression::UnaryOperator { op, exp } => {
            eval_unary_op_expr(exp.as_ref(), *op, scope).map(ReturnValue::from)
        }
        Expression::BinaryOperator { lhs, op, rhs } => {
            binary_op_eval(*op, lhs, rhs, scope).map(ReturnValue::from)
        }
    }
}

#[cfg(test)]
mod test {
    use crate as ast_vm;
    use crate::{
        lang::{Context, ReturnValue},
        LuaError,
    };
    use luar_lex::{NumberLiteral, StringLiteral, Token};
    use luar_syn::{lua_parser, unspanned_lua_token_parser};
    use test_util::Finite;

    #[test]
    fn eval_nil() -> Result<(), LuaError> {
        let module = lua_parser::module("return nil")?;
        let mut context = Context::new();
        assert_eq!(
            ast_vm::eval_module(&module, &mut context)?,
            ReturnValue::NIL
        );
        Ok(())
    }

    #[quickcheck]
    fn eval_number_literal(Finite(num): Finite<f64>) -> Result<(), LuaError> {
        let module =
            unspanned_lua_token_parser::module([Token::Return, Token::Number(NumberLiteral(num))])?;
        let mut context = Context::new();
        assert!(ast_vm::eval_module(&module, &mut context)?.total_eq(&ReturnValue::number(num)));
        Ok(())
    }

    #[quickcheck]
    fn eval_string_literal(str: String) -> Result<(), LuaError> {
        let module = unspanned_lua_token_parser::module([
            Token::Return,
            Token::String(StringLiteral(str.clone())),
        ])?;
        let mut context = Context::new();
        assert_eq!(
            ast_vm::eval_module(&module, &mut context)?,
            ReturnValue::string(str)
        );
        Ok(())
    }
}
