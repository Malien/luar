use super::{EvalContext, Result};
use crate::{
    expr::binary_op::*,
    lang::LuaValue,
    opt::{eval::eval_expr, syn::Expression},
    EvalError,
};
use luar_error::ArithmeticOperator;
use luar_syn::BinaryOperator;

pub(crate) fn binary_op_eval(
    op: luar_syn::BinaryOperator,
    lhs: &Expression,
    rhs: &Expression,
    ctx: &mut EvalContext<'_>,
) -> Result<LuaValue> {
    use BinaryOperator::*;

    let lhs = eval_expr(lhs, ctx)?.first_value();
    match op {
        And if lhs.is_falsy() => return Ok(lhs),
        Or if lhs.is_truthy() => return Ok(lhs),
        _ => {}
    };

    let rhs = eval_expr(rhs, ctx)?.first_value();
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
