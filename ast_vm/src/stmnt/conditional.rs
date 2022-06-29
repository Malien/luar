use crate::{
    eval_block, eval_expr,
    lang::{LocalScope, ScopeHolder},
    ControlFlow, EvalError,
};
use luar_syn::{Conditional, ConditionalTail};

pub(crate) fn eval_conditional(
    conditional: &Conditional,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ControlFlow, EvalError> {
    let Conditional {
        condition,
        body,
        tail,
    } = conditional;

    if eval_expr(condition, scope)?.first_value().is_truthy() {
        eval_block(body, &mut scope.child_scope())
    } else {
        match tail {
            ConditionalTail::End => Ok(ControlFlow::Continue),
            ConditionalTail::Else(block) => eval_block(block, &mut scope.child_scope()),
            ConditionalTail::ElseIf(condition) => eval_conditional(condition, scope),
        }
    }
}
