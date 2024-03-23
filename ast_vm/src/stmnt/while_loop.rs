use crate::{
    eval_block, eval_expr,
    lang::{LocalScope, ScopeHolder},
    ControlFlow,
};
use luar_syn::WhileLoop;

pub(crate) fn eval_while_loop(
    while_loop: &WhileLoop,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ControlFlow, crate::EvalError> {
    let WhileLoop { condition, body } = while_loop;
    while eval_expr(condition, scope)?.first_value().is_truthy() {
        if let ControlFlow::Return(ret_value) = eval_block(body, &mut scope.child_scope())? {
            return Ok(ControlFlow::Return(ret_value));
        }
    }
    Ok(ControlFlow::Continue)
}
