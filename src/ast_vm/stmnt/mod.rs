use super::{eval_fn_call, ControlFlow};
use crate::{
    lang::{EvalError, LocalScope, ScopeHolder},
    syn::Statement,
};

mod assignment;
pub(crate) use assignment::*;
mod conditional;
pub(crate) use conditional::*;
mod local_decl;
pub(crate) use local_decl::*;
mod while_loop;
pub(crate) use while_loop::*;

pub(crate) fn eval_stmnt(
    stmnt: &Statement,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ControlFlow, EvalError> {
    use Statement::*;
    match stmnt {
        Assignment(assignment) => eval_assignment(assignment, scope).map(|_| ControlFlow::Continue),
        LocalDeclaration(decl) => eval_decl(decl, scope).map(|_| ControlFlow::Continue),
        FunctionCall(func_call) => eval_fn_call(func_call, scope).map(|_| ControlFlow::Continue),
        If(conditional) => eval_conditional(conditional, scope),
        While(while_loop) => eval_while_loop(while_loop, scope),
        _ => todo!(),
    }
}
