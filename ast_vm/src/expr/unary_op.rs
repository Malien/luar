use super::eval_expr;
use crate::{
    lang::{LocalScope, LuaValue, ScopeHolder},
    ArithmeticError, EvalError, TypeError,
};
use luar_syn::{Expression, UnaryOperator};

pub(crate) fn eval_unary_op_expr(
    expr: &Expression,
    op: UnaryOperator,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<LuaValue, EvalError> {
    eval_expr(expr, scope).and_then(|value| {
        unary_op_eval(op, value.first_value())
            .map_err(TypeError::Arithmetic)
            .map_err(EvalError::from)
    })
}

fn unary_op_eval(op: UnaryOperator, value: LuaValue) -> Result<LuaValue, ArithmeticError> {
    match op {
        UnaryOperator::Minus => unary_minus_eval(value),
        UnaryOperator::Not if value.is_falsy() => Ok(LuaValue::true_value()),
        UnaryOperator::Not => Ok(LuaValue::Nil),
    }
}

fn unary_minus_eval(value: LuaValue) -> Result<LuaValue, ArithmeticError> {
    match value.as_number() {
        Some(num) => Ok(LuaValue::number(-num.as_f64())),
        None => Err(ArithmeticError::UnaryMinus(value)),
    }
}

#[cfg(test)]
mod test {
    mod expressions {
        use luar_error::assert_type_error;
        use luar_lex::{NumberLiteral, StringLiteral};
        use luar_syn::{lua_parser, Expression, UnaryOperator};
        use quickcheck::Arbitrary;
        use test_util::Finite;

        use crate as ast_vm;
        use crate::{
            lang::{GlobalContext, ReturnValue, ScopeHolder},
            util::eq_with_nan,
            ArithmeticError, EvalError, LuaError, TypeError,
        };

        fn negation_expr(num: f64) -> Expression {
            Expression::UnaryOperator {
                op: UnaryOperator::Minus,
                exp: Box::new(Expression::Number(NumberLiteral(num))),
            }
        }

        #[quickcheck]
        fn eval_negation(Finite(num): Finite<f64>) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(num);
            let res = ast_vm::eval_expr(&expr, &mut context.top_level_scope())?
                .assert_single()
                .unwrap_number()
                .as_f64();
            assert_eq!(res, -num);
            Ok(())
        }

        #[test]
        fn eval_negation_on_nan() -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(f64::NAN);
            let res = ast_vm::eval_expr(&expr, &mut context.top_level_scope())?
                .assert_single()
                .unwrap_number()
                .as_f64();
            assert!(eq_with_nan(res, f64::NAN));
            Ok(())
        }

        #[test]
        fn eval_negation_on_inf() -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(f64::INFINITY);
            let res = ast_vm::eval_expr(&expr, &mut context.top_level_scope())?
                .assert_single()
                .unwrap_number()
                .as_f64();
            assert!(eq_with_nan(res, f64::NEG_INFINITY));
            Ok(())
        }

        #[test]
        fn eval_negation_on_neg_inf() -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(f64::NEG_INFINITY);
            let res = ast_vm::eval_expr(&expr, &mut context.top_level_scope())?
                .assert_single()
                .unwrap_number()
                .as_f64();
            assert!(eq_with_nan(res, f64::INFINITY));
            Ok(())
        }

        #[quickcheck]
        fn eval_negation_on_convertible_str(num: f64) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = Expression::UnaryOperator {
                op: UnaryOperator::Minus,
                exp: Box::new(Expression::String(StringLiteral(format!("{}", num)))),
            };
            let res = ast_vm::eval_expr(&expr, &mut context.top_level_scope())?
                .assert_single()
                .unwrap_number()
                .as_f64();
            assert!(eq_with_nan(res, -num));
            Ok(())
        }

        #[test]
        fn eval_unary_minus_on_unsupported_type_errors() {
            let mut context = GlobalContext::new();
            let unsupported = [
                Expression::Nil,
                Expression::String(StringLiteral("Definitely not a number".to_string())),
                // syn::Expression::TableConstructor(TableConstructor::empty()),
            ];

            for exp in unsupported {
                let expr = Expression::UnaryOperator {
                    op: UnaryOperator::Minus,
                    exp: Box::new(exp),
                };
                let res = ast_vm::eval_expr(&expr, &mut context.top_level_scope());
                assert_type_error!(TypeError::Arithmetic(ArithmeticError::UnaryMinus(_)), res);
            }
        }

        #[test]
        fn eval_not_on_nil() -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let expr = lua_parser::expression("not nil")?;
            assert_eq!(
                ast_vm::eval_expr(&expr, &mut context.top_level_scope())?,
                ReturnValue::true_value()
            );
            Ok(())
        }

        #[derive(Debug, Clone)]
        pub struct TruthyExpression(Expression);

        impl Arbitrary for TruthyExpression {
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                Self(match u8::arbitrary(g) % 2 {
                    0 => Expression::Number(Arbitrary::arbitrary(g)),
                    1 => Expression::String(Arbitrary::arbitrary(g)),
                    _ => unreachable!(),
                })
            }

            fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                Box::new(self.0.shrink().map(Self))
            }
        }

        #[quickcheck]
        fn eval_not_on_truthy_vals(
            TruthyExpression(expr): TruthyExpression,
        ) -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let expr = Expression::UnaryOperator {
                op: UnaryOperator::Not,
                exp: Box::new(expr),
            };
            assert_eq!(
                ast_vm::eval_expr(&expr, &mut context.top_level_scope())?,
                ReturnValue::Nil
            );
            Ok(())
        }
    }
}
