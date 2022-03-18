use crate::{
    lang::{
        ArithmeticError, ArithmeticOperator, Eval, EvalContext, EvalError, LuaValue,
        OrderingOperator, ReturnValue, TableRef, TypeError,
    },
    lex::{NumberLiteral, StringLiteral},
    syn::{BinaryOperator, Expression, UnaryOperator},
};

impl Eval for Expression {
    type Return = ReturnValue;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        match self {
            Expression::Nil => Ok(ReturnValue::Nil),
            Expression::Number(NumberLiteral(num)) => Ok(ReturnValue::Number((*num).into())),
            Expression::String(StringLiteral(str)) => Ok(ReturnValue::String(str.clone())),
            Expression::Variable(var) => var.eval(context).map(ReturnValue::from),
            Expression::TableConstructor(tbl) => tbl
                .eval(context)
                .map(TableRef::from)
                .map(ReturnValue::Table),
            Expression::FunctionCall(call) => call.eval(context),
            Expression::UnaryOperator { op, exp } => {
                eval_unary_op_expr(exp.as_ref(), *op, context).map(ReturnValue::from)
            }
            Expression::BinaryOperator { lhs, op, rhs } => {
                binary_op_eval(*op, lhs, rhs, context).map(ReturnValue::from)
            }
        }
    }
}

fn eval_unary_op_expr<Context>(
    expr: &Expression,
    op: UnaryOperator,
    context: &mut Context,
) -> Result<LuaValue, EvalError>
where
    Context: EvalContext + ?Sized,
{
    expr.eval(context).and_then(|value| {
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

fn binary_op_eval<Context>(
    op: BinaryOperator,
    lhs: &Expression,
    rhs: &Expression,
    context: &mut Context,
) -> Result<LuaValue, EvalError>
where
    Context: EvalContext + ?Sized,
{
    use BinaryOperator::*;

    let lhs = lhs.eval(context)?.first_value();
    match op {
        And if lhs.is_falsy() => return Ok(lhs),
        And => return rhs.eval(context).map(ReturnValue::first_value),
        Or if lhs.is_truthy() => return Ok(lhs),
        Or => return rhs.eval(context).map(ReturnValue::first_value),
        _ => {}
    };
    let rhs = rhs.eval(context)?.first_value();

    match op {
        Equals => return Ok(LuaValue::from_bool(lhs == rhs)),
        NotEquals => return Ok(LuaValue::from_bool(lhs != rhs)),
        _ => {}
    }

    match op {
        Less => less_than(lhs, rhs),
        Greater => greater_than(lhs, rhs),
        LessOrEquals => less_or_equals(lhs, rhs),
        GreaterOrEquals => greater_or_equals(lhs, rhs),
        Plus => binary_number_op(lhs, rhs, ArithmeticOperator::Add, std::ops::Add::add),
        Minus => binary_number_op(lhs, rhs, ArithmeticOperator::Sub, std::ops::Sub::sub),
        Mul | Div | Exp | Concat => todo!(),
        And | Or | Equals | NotEquals => unreachable!(),
    }
    .map_err(EvalError::TypeError)
}

fn binary_number_op(
    lhs: LuaValue,
    rhs: LuaValue,
    op: ArithmeticOperator,
    op_fn: impl FnOnce(f64, f64) -> f64,
) -> Result<LuaValue, TypeError> {
    if let (Some(lhs), Some(rhs)) = (lhs.as_number(), rhs.as_number()) {
        let res = op_fn(lhs.as_f64(), rhs.as_f64());
        Ok(LuaValue::number(res))
    } else {
        Err(TypeError::Arithmetic(ArithmeticError::Binary {
            lhs,
            rhs,
            op,
        }))
    }
}

macro_rules! ord_op {
    ($name: ident, $cmp_op: tt, $op: expr) => {
        fn $name(lhs: LuaValue, rhs: LuaValue) -> Result<LuaValue, TypeError> {
            match (&lhs, &rhs) {
                (LuaValue::Number(lhs), LuaValue::Number(rhs)) =>
                    Ok(LuaValue::from_bool(lhs $cmp_op rhs)),
                (LuaValue::String(lhs), LuaValue::String(rhs)) =>
                    Ok(LuaValue::from_bool(lhs $cmp_op rhs)),
                (LuaValue::Number(lhs), LuaValue::String(rhs)) =>
                    Ok(LuaValue::from_bool(&format!("{}", lhs) $cmp_op rhs)),
                (LuaValue::String(lhs), LuaValue::Number(rhs)) =>
                    Ok(LuaValue::from_bool(lhs $cmp_op &format!("{}", rhs))),
                _ => Err(TypeError::Ordering {
                    lhs,
                    rhs,
                    op: $op
                })
            }
        }
    };
}

ord_op!(less_than, <, OrderingOperator::Less);
ord_op!(greater_than, >, OrderingOperator::Greater);
ord_op!(less_or_equals, <=, OrderingOperator::LessOrEquals);
ord_op!(greater_or_equals, >=, OrderingOperator::GreaterOrEquals);

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use crate::{
        error::LuaError,
        lang::{
            Eval, EvalContextExt, GlobalContext, LuaFunction, LuaNumber, LuaValue, ReturnValue,
        },
        lex::{NumberLiteral, StringLiteral, Token},
        ne_vec,
        syn::{lua_parser, unspanned_lua_token_parser},
        test_util::Finite,
        util::NonEmptyVec,
    };

    mod expressions {
        use quickcheck::Arbitrary;

        use crate::{
            error::LuaError,
            lang::{ArithmeticError, Eval, EvalError, GlobalContext, ReturnValue, TypeError},
            lex::{NumberLiteral, StringLiteral},
            syn,
            test_util::Finite,
            util::eq_with_nan,
        };

        #[test]
        fn eval_nil() -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::Nil;
            assert_eq!(expr.eval(&mut context)?, ReturnValue::Nil);
            Ok(())
        }

        #[quickcheck]
        fn eval_number_literal(num: f64) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::Number(NumberLiteral(num));
            let res = expr
                .eval(&mut context)?
                .assert_single()
                .unwrap_number()
                .as_f64();
            assert!(eq_with_nan(res, num));
            Ok(())
        }

        #[quickcheck]
        fn eval_string_literal(str: String) -> Result<(), EvalError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::String(StringLiteral(str.clone()));
            assert_eq!(
                expr.eval(&mut context)?.assert_single().unwrap_string(),
                str
            );
            Ok(())
        }

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
            let res = expr
                .eval(&mut context)?
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
            let res = expr
                .eval(&mut context)?
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
            let res = expr
                .eval(&mut context)?
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
            let res = expr
                .eval(&mut context)?
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
            let res = expr
                .eval(&mut context)?
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
                let res = expr.eval(&mut context);
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
            let exp = syn::lua_parser::expression("not nil")?;
            assert_eq!(exp.eval(&mut context)?, ReturnValue::number(1));
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
            assert_eq!(expr.eval(&mut context)?, ReturnValue::Nil);
            Ok(())
        }

        #[derive(Debug, Clone)]
        pub struct SimpleExpression(syn::Expression);

        impl Arbitrary for SimpleExpression {
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                Self(match u8::arbitrary(g) % 3 {
                    0 => syn::Expression::Nil,
                    1 => syn::Expression::Number(Arbitrary::arbitrary(g)),
                    2 => syn::Expression::String(Arbitrary::arbitrary(g)),
                    _ => unreachable!(),
                })
            }

            fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
                Box::new(self.0.shrink().map(Self))
            }
        }

        #[quickcheck]
        fn eval_and_on_truthy(
            TruthyExpression(lhs): TruthyExpression,
            SimpleExpression(rhs): SimpleExpression,
        ) -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let expected = rhs.eval(&mut context)?;
            let expr = syn::Expression::BinaryOperator {
                op: syn::BinaryOperator::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };

            assert!(expr.eval(&mut context)?.total_eq(&expected));
            Ok(())
        }

        #[quickcheck]
        fn eval_and_on_falsy(SimpleExpression(rhs): SimpleExpression) -> Result<(), LuaError> {
            let mut context = GlobalContext::new();
            let expr = syn::Expression::BinaryOperator {
                op: syn::BinaryOperator::And,
                lhs: Box::new(syn::Expression::Nil),
                rhs: Box::new(rhs),
            };

            assert_eq!(expr.eval(&mut context)?, ReturnValue::Nil);
            Ok(())
        }
    }

    #[test]
    fn eval_nil() -> Result<(), LuaError> {
        let module = lua_parser::module("return nil")?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, ReturnValue::Nil);
        Ok(())
    }

    #[quickcheck]
    fn eval_number_literal(Finite(num): Finite<f64>) -> Result<(), LuaError> {
        let module =
            unspanned_lua_token_parser::module([Token::Return, Token::Number(NumberLiteral(num))])?;
        let mut context = GlobalContext::new();
        assert!(module
            .eval(&mut context)?
            .total_eq(&ReturnValue::number(num)));
        Ok(())
    }

    #[quickcheck]
    fn eval_string_literal(str: String) -> Result<(), LuaError> {
        let module = unspanned_lua_token_parser::module([
            Token::Return,
            Token::String(StringLiteral(str.clone())),
        ])?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, ReturnValue::String(str));
        Ok(())
    }

    #[quickcheck]
    fn eval_and_truthy(lhs: LuaValue, rhs: LuaValue) -> Result<TestResult, LuaError> {
        if lhs.is_falsy() {
            return Ok(TestResult::discard());
        }
        let module = lua_parser::module("return lhs and rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs.clone());
        let res = module.eval(&mut context)?;
        assert!(LuaValue::total_eq(&res.assert_single(), &rhs));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_and_falsy(rhs: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module("return lhs and rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", LuaValue::Nil);
        context.set("rhs", rhs.clone());
        let res = module.eval(&mut context)?;
        assert_eq!(res, ReturnValue::Nil);
        Ok(())
    }

    #[quickcheck]
    fn multiple_return_is_not_propagated_in_and(
        values: NonEmptyVec<LuaValue>,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return 1 and mult()")?;
        let mut context = GlobalContext::new();
        let ret_value = ReturnValue::MultiValue(values.clone());
        let mult_fn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
        context.set("mult", LuaValue::Function(mult_fn));
        let res = module.eval(&mut context)?;
        assert!(ReturnValue::total_eq(&res, &values.move_first().into()));
        Ok(())
    }

    #[test]
    #[ignore = "This relies on fn def, fn call and assignment which are not implemented yet"]
    fn and_short_circuits() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "side_effect_committed = nil

            function side_effecty_fn()
                side_effect_committed = 1
            end

            res = nil and side_effecty_fn()
            return side_effect_committed",
        )?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, ReturnValue::Nil);
        Ok(())
    }

    #[quickcheck]
    fn eval_or_truthy(lhs: LuaValue, rhs: LuaValue) -> Result<TestResult, LuaError> {
        if lhs.is_falsy() {
            return Ok(TestResult::discard());
        }
        let module = lua_parser::module("return lhs or rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs.clone());
        context.set("rhs", rhs);
        let res = module.eval(&mut context)?;
        assert!(LuaValue::total_eq(&res.assert_single(), &lhs));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_or_falsy(rhs: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module("return lhs or rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", LuaValue::Nil);
        context.set("rhs", rhs.clone());
        let res = module.eval(&mut context)?;
        assert!(LuaValue::total_eq(&res.assert_single(), &rhs));
        Ok(())
    }

    #[quickcheck]
    fn multiple_return_is_not_propagated_in_or(
        values: NonEmptyVec<LuaValue>,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return nil or mult()")?;
        let mut context = GlobalContext::new();
        let ret_value = ReturnValue::MultiValue(values.clone());
        let mult_fn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
        context.set("mult", LuaValue::Function(mult_fn));
        let res = module.eval(&mut context)?;
        assert!(ReturnValue::total_eq(&res, &values.move_first().into()));
        Ok(())
    }

    #[test]
    #[ignore = "This relies on fn def, fn call and assignment which are not implemented yet"]
    fn or_short_circuits() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "
            side_effect_committed = nil

            function side_effecty_fn()
                side_effect_committed = 1
            end

            res = 1 or side_effecty_fn()
            return side_effect_committed
        ",
        )?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, ReturnValue::Nil);
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

        let module = lua_parser::module("return value == value")?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
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
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs);
        let res = module.eval(&mut context)?;
        assert_eq!(expected, res.assert_single());
        Ok(())
    }

    #[quickcheck]
    fn not_equals_is_the_negation_of_equality(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return (not (lhs ~= rhs)) == (lhs == rhs)")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs);
        let res = module.eval(&mut context)?;
        assert_eq!(LuaValue::true_value(), res.assert_single());
        Ok(())
    }

    #[quickcheck]
    fn binary_plus_on_convertible_values_is_the_sum_of_those_values(
        lhs: f64,
        rhs: f64,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return lhs + rhs")?;
        let mut context = GlobalContext::new();

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::number(rhs));
        let res = module.eval(&mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs + rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::number(rhs));
        let res = module.eval(&mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs + rhs)));

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = module.eval(&mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs + rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = module.eval(&mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs + rhs)));

        Ok(())
    }

    #[quickcheck]
    fn binary_plus_on_incompatible_types_is_not_supported(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if let (Some(_), Some(_)) = (lhs.as_number(), rhs.as_number()) {
            return Ok(TestResult::discard());
        }
        let module = lua_parser::module("return lhs + rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs);
        let res = module.eval(&mut context);
        assert!(res.is_err());

        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn binary_minus_on_convertible_values_is_the_difference_of_those_values(
        lhs: f64,
        rhs: f64,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return lhs - rhs")?;
        let mut context = GlobalContext::new();

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::number(rhs));
        let res = module.eval(&mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs - rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::number(rhs));
        let res = module.eval(&mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs - rhs)));

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = module.eval(&mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs - rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = module.eval(&mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs - rhs)));

        Ok(())
    }

    #[quickcheck]
    fn binary_minus_on_incompatible_types_is_not_supported(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if let (Some(_), Some(_)) = (lhs.as_number(), rhs.as_number()) {
            return Ok(TestResult::discard());
        }
        let module = lua_parser::module("return lhs - rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs);
        let res = module.eval(&mut context);
        assert!(res.is_err());

        Ok(TestResult::passed())
    }

    #[quickcheck]
    #[allow(non_snake_case)]
    fn comparing_numbers_behave_according_to_IEEE754(lhs: f64, rhs: f64) -> Result<(), LuaError> {
        let module = lua_parser::module("return a > b, a < b, a >= b, a <= b")?;
        let mut context = GlobalContext::new();
        context.set("a", LuaValue::number(lhs));
        context.set("b", LuaValue::number(rhs));
        let expected = ne_vec![
            LuaValue::from_bool(lhs > rhs),
            LuaValue::from_bool(lhs < rhs),
            LuaValue::from_bool(lhs >= rhs),
            LuaValue::from_bool(lhs <= rhs)
        ];
        let res = module.eval(&mut context)?;
        assert_eq!(res, ReturnValue::MultiValue(expected));
        Ok(())
    }

    #[quickcheck]
    fn comparing_strings_orders_then_lexicographically(
        lhs: String,
        rhs: String,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return a > b, a < b, a >= b, a <= b")?;
        let mut context = GlobalContext::new();
        context.set("a", LuaValue::string(lhs.clone()));
        context.set("b", LuaValue::string(rhs.clone()));
        let expected = ne_vec![
            LuaValue::from_bool(lhs > rhs),
            LuaValue::from_bool(lhs < rhs),
            LuaValue::from_bool(lhs >= rhs),
            LuaValue::from_bool(lhs <= rhs)
        ];
        let res = module.eval(&mut context)?;
        assert_eq!(res, ReturnValue::MultiValue(expected));
        Ok(())
    }

    #[quickcheck]
    fn comparing_strings_and_numbers_coerces_numbers_to_strings(
        str: String,
        num: LuaNumber,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return a > b, a < b, a >= b, a <= b")?;
        let mut context = GlobalContext::new();

        {
            context.set("a", LuaValue::string(str.clone()));
            context.set("b", LuaValue::number(num));
            let lhs = &str;
            let rhs = &format!("{}", num);
            let expected = ne_vec![
                LuaValue::from_bool(lhs > rhs),
                LuaValue::from_bool(lhs < rhs),
                LuaValue::from_bool(lhs >= rhs),
                LuaValue::from_bool(lhs <= rhs)
            ];
            let res = module.eval(&mut context)?;
            assert_eq!(res, ReturnValue::MultiValue(expected));
        }
        {
            context.set("a", LuaValue::number(num));
            context.set("b", LuaValue::string(str.clone()));
            let lhs = &format!("{}", num);
            let rhs = &str;
            let expected = ne_vec![
                LuaValue::from_bool(lhs > rhs),
                LuaValue::from_bool(lhs < rhs),
                LuaValue::from_bool(lhs >= rhs),
                LuaValue::from_bool(lhs <= rhs)
            ];
            let res = module.eval(&mut context)?;
            assert_eq!(res, ReturnValue::MultiValue(expected));
        }

        Ok(())
    }

    #[quickcheck]
    fn values_other_than_numbers_and_strings_are_not_comparable(
        val: LuaValue,
    ) -> Result<(), LuaError> {
        let ops = [">", "<", ">=", "<="];
        let modules = IntoIterator::into_iter(ops)
            .flat_map(|op| {
                [
                    format!("return 1 {} value", op),
                    format!("return value {} 1", op),
                ]
            })
            .map(|str| lua_parser::module(&str))
            .collect::<Result<Vec<_>, _>>()?;
        let mut context = GlobalContext::new();
        let is_comparable = val.is_comparable();
        context.set("value", val);

        for module in modules {
            let res = module.eval(&mut context);
            if is_comparable {
                assert!(res.is_ok());
            } else {
                assert!(res.is_err());
            }
        }

        Ok(())
    }
}
