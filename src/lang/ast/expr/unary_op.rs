use crate::{syn::{Expression, UnaryOperator}, lang::{LocalScope, ScopeHolder, LuaValue, EvalError, TypeError, ArithmeticError}};

use super::eval_expr;

pub(crate) fn eval_unary_op_expr(
    expr: &Expression,
    op: UnaryOperator,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<LuaValue, EvalError> {
    eval_expr(expr, scope).and_then(|value| {
        unary_op_eval(op, value.first_value())
            .map_err(TypeError::Arithmetic)
            .map_err(EvalError::TypeError)
    })
}

fn unary_op_eval(op: UnaryOperator, value: LuaValue) -> Result<LuaValue, ArithmeticError> {
    match op {
        UnaryOperator::Minus => unary_minus_eval(value),
        UnaryOperator::Not if value.is_falsy() => Ok(LuaValue::number(1)),
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
        use quickcheck::Arbitrary;

        use crate::{
            error::LuaError,
            lang::{
                ast, ArithmeticError, EvalError, GlobalContext, ReturnValue, ScopeHolder, TypeError,
            },
            lex::{NumberLiteral, StringLiteral},
            syn,
            test_util::Finite,
            util::eq_with_nan,
        };

        fn negation_expr(num: f64) -> syn::Expression {
            syn::Expression::UnaryOperator {
                op: syn::UnaryOperator::Minus,
                exp: Box::new(syn::Expression::Number(NumberLiteral(num))),
            }
        }

        #[quickcheck]
        fn eval_negation(Finite(num): Finite<f64>) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = negation_expr(num);
            let res = ast::eval_expr(&expr, &mut context.top_level_scope())?
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
            let res = ast::eval_expr(&expr, &mut context.top_level_scope())?
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
            let res = ast::eval_expr(&expr, &mut context.top_level_scope())?
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
            let res = ast::eval_expr(&expr, &mut context.top_level_scope())?
                .assert_single()
                .unwrap_number()
                .as_f64();
            assert!(eq_with_nan(res, f64::INFINITY));
            Ok(())
        }

        #[quickcheck]
        fn eval_negation_on_convertible_str(num: f64) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::UnaryOperator {
                op: syn::UnaryOperator::Minus,
                exp: Box::new(syn::Expression::String(StringLiteral(format!("{}", num)))),
            };
            let res = ast::eval_expr(&expr, &mut context.top_level_scope())?
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
                syn::Expression::Nil,
                syn::Expression::String(StringLiteral("Definitely not a number".to_string())),
                // syn::Expression::TableConstructor(TableConstructor::empty()),
            ];

            for exp in unsupported {
                let expr = syn::Expression::UnaryOperator {
                    op: syn::UnaryOperator::Minus,
                    exp: Box::new(exp),
                };
                let res = ast::eval_expr(&expr, &mut context.top_level_scope());
                assert!(matches!(
                    res,
                    Err(EvalError::TypeError(TypeError::Arithmetic(
                        ArithmeticError::UnaryMinus(_)
                    )))
                ));
            }
        }

        #[test]
        fn eval_not_on_nil() -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let expr = syn::lua_parser::expression("not nil")?;
            assert_eq!(
                ast::eval_expr(&expr, &mut context.top_level_scope())?,
                ReturnValue::number(1)
            );
            Ok(())
        }

        #[derive(Debug, Clone)]
        pub struct TruthyExpression(syn::Expression);

        impl Arbitrary for TruthyExpression {
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                Self(match u8::arbitrary(g) % 2 {
                    0 => syn::Expression::Number(Arbitrary::arbitrary(g)),
                    1 => syn::Expression::String(Arbitrary::arbitrary(g)),
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
            let expr = syn::Expression::UnaryOperator {
                op: syn::UnaryOperator::Not,
                exp: Box::new(expr),
            };
            assert_eq!(
                ast::eval_expr(&expr, &mut context.top_level_scope())?,
                ReturnValue::Nil
            );
            Ok(())
        }
    }
}
