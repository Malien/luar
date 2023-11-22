use crate::{
    eval_expr,
    lang::{LocalScope, LuaValue, ScopeHolder},
    ArithmeticError, EvalError, TypeError,
};
use luar_error::{ArithmeticOperator, OrderingOperator};
use luar_syn::{BinaryOperator, Expression};

pub(crate) fn binary_op_eval(
    op: BinaryOperator,
    lhs: &Expression,
    rhs: &Expression,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<LuaValue, EvalError> {
    use BinaryOperator::*;

    let lhs = eval_expr(lhs, scope)?.first_value();
    match op {
        And if lhs.is_falsy() => return Ok(lhs),
        Or if lhs.is_truthy() => return Ok(lhs),
        _ => {}
    };

    let rhs = eval_expr(rhs, scope)?.first_value();
    match op {
        Equals => return Ok(LuaValue::from_bool(lhs == rhs)),
        NotEquals => return Ok(LuaValue::from_bool(lhs != rhs)),
        And | Or => return Ok(rhs),
        _ => {}
    }

    match op {
        Less => less_than(lhs, rhs),
        Greater => greater_than(lhs, rhs),
        LessOrEquals => less_or_equals(lhs, rhs),
        GreaterOrEquals => greater_or_equals(lhs, rhs),
        Plus => binary_number_op(lhs, rhs, ArithmeticOperator::Add, std::ops::Add::add),
        Minus => binary_number_op(lhs, rhs, ArithmeticOperator::Sub, std::ops::Sub::sub),
        Mul => binary_number_op(lhs, rhs, ArithmeticOperator::Mul, std::ops::Mul::mul),
        Div => binary_number_op(lhs, rhs, ArithmeticOperator::Div, std::ops::Div::div),
        Exp => todo!("No support for ^ operator yet."),
        Concat => concat(lhs, rhs),
        And | Or | Equals | NotEquals => unreachable!(),
    }
    .map_err(EvalError::from)
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
                    op: Some($op)
                })
            }
        }
    };
}

ord_op!(less_than, <, OrderingOperator::Less);
ord_op!(greater_than, >, OrderingOperator::Greater);
ord_op!(less_or_equals, <=, OrderingOperator::LessOrEquals);
ord_op!(greater_or_equals, >=, OrderingOperator::GreaterOrEquals);

fn concat(lhs: LuaValue, rhs: LuaValue) -> Result<LuaValue, TypeError> {
    match (&lhs, &rhs) {
        (LuaValue::String(lhs), LuaValue::String(rhs)) => {
            Ok(LuaValue::String(format!("{lhs}{rhs}")))
        }
        (LuaValue::Number(lhs), LuaValue::String(rhs)) => {
            Ok(LuaValue::String(format!("{lhs}{rhs}")))
        }
        (LuaValue::String(lhs), LuaValue::Number(rhs)) => {
            Ok(LuaValue::String(format!("{lhs}{rhs}")))
        }
        (LuaValue::Number(lhs), LuaValue::Number(rhs)) => {
            Ok(LuaValue::String(format!("{lhs}{rhs}")))
        }
        _ => Err(TypeError::StringConcat { lhs, rhs }),
    }
}

#[cfg(test)]
mod test {
    use crate as ast_vm;
    use crate::{
        lang::{GlobalContext, LuaFunction, LuaValue, ReturnValue},
        LuaError,
    };
    use luar_syn::lua_parser;
    use non_empty::NonEmptyVec;
    use quickcheck::TestResult;

    #[quickcheck]
    fn eval_and_truthy(lhs: LuaValue, rhs: LuaValue) -> Result<TestResult, LuaError> {
        if lhs.is_falsy() {
            return Ok(TestResult::discard());
        }
        let module = lua_parser::module("return lhs and rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs.clone());
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(LuaValue::total_eq(&res.assert_single(), &rhs));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_and_falsy(rhs: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module("return lhs and rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", LuaValue::Nil);
        context.set("rhs", rhs.clone());
        let res = ast_vm::eval_module(&module, &mut context)?;
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
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(ReturnValue::total_eq(&res, &values.move_first().into()));
        Ok(())
    }

    #[test]
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
        assert_eq!(
            ast_vm::eval_module(&module, &mut context)?,
            ReturnValue::Nil
        );
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
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(LuaValue::total_eq(&res.assert_single(), &lhs));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_or_falsy(rhs: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module("return lhs or rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", LuaValue::Nil);
        context.set("rhs", rhs.clone());
        let res = ast_vm::eval_module(&module, &mut context)?;
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
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(ReturnValue::total_eq(&res, &values.move_first().into()));
        Ok(())
    }

    #[test]
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
        assert_eq!(
            ast_vm::eval_module(&module, &mut context)?,
            ReturnValue::Nil
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

        let module = lua_parser::module("return value == value")?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
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
        let res = ast_vm::eval_module(&module, &mut context)?;
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
        let res = ast_vm::eval_module(&module, &mut context)?;
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
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs + rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::number(rhs));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs + rhs)));

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs + rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = ast_vm::eval_module(&module, &mut context)?;
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
        let res = ast_vm::eval_module(&module, &mut context);
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
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs - rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::number(rhs));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs - rhs)));

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs - rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = ast_vm::eval_module(&module, &mut context)?;
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
        let res = ast_vm::eval_module(&module, &mut context);
        assert!(res.is_err());

        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn multiplication_on_convertible_values_is_the_product_of_those_values(
        lhs: f64,
        rhs: f64,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return lhs * rhs")?;
        let mut context = GlobalContext::new();

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::number(rhs));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs * rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::number(rhs));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs * rhs)));

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs * rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs * rhs)));

        Ok(())
    }

    #[quickcheck]
    fn multiplication_on_incompatible_types_is_not_supported(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if let (Some(_), Some(_)) = (lhs.as_number(), rhs.as_number()) {
            return Ok(TestResult::discard());
        }
        let module = lua_parser::module("return lhs * rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs);
        let res = ast_vm::eval_module(&module, &mut context);
        assert!(res.is_err());

        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn division_of_convertible_values_is_the_division_of_those_values(
        lhs: f64,
        rhs: f64,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module("return lhs / rhs")?;
        let mut context = GlobalContext::new();

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::number(rhs));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs / rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::number(rhs));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs / rhs)));

        context.set("lhs", LuaValue::number(lhs));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs / rhs)));

        context.set("lhs", LuaValue::String(lhs.to_string()));
        context.set("rhs", LuaValue::String(rhs.to_string()));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.total_eq(&ReturnValue::number(lhs / rhs)));

        Ok(())
    }

    #[quickcheck]
    fn division_on_incompatible_types_is_not_supported(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if let (Some(_), Some(_)) = (lhs.as_number(), rhs.as_number()) {
            return Ok(TestResult::discard());
        }
        let module = lua_parser::module("return lhs / rhs")?;
        let mut context = GlobalContext::new();
        context.set("lhs", lhs);
        context.set("rhs", rhs);
        let res = ast_vm::eval_module(&module, &mut context);
        assert!(res.is_err());

        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn concat(lhs: LuaValue, rhs: LuaValue) {
        let module = lua_parser::module("return lhs .. rhs").unwrap();
        let mut context = GlobalContext::new();
        context.set("lhs", lhs.clone());
        context.set("rhs", rhs.clone());

        let res = ast_vm::eval_module(&module, &mut context);
        if let (Some(lhs), Some(rhs)) = (lhs.coerce_to_string(), rhs.coerce_to_string()) {
            let res = res.unwrap();
            assert!(res.total_eq(&ReturnValue::string(lhs + &rhs)));
        } else {
            assert!(res.is_err());
        }
    }
}
