use super::{eval_fn_decl, eval_ret, eval_stmnt, ControlFlow};
use crate::{lang::{Context, LocalScope, ReturnValue, ScopeHolder}, EvalError};
use luar_syn::{Chunk, Module};

pub fn eval_module(module: &Module, context: &mut Context) -> Result<ReturnValue, EvalError> {
    let mut scope = context.top_level_scope();
    for chunk in &*module.chunks {
        if let ControlFlow::Return(value) = eval_chunk(chunk, &mut scope)? {
            return Ok(value);
        }
    }
    match module.ret {
        Some(ref ret) => eval_ret(ret, &mut scope),
        None => Ok(ReturnValue::NIL),
    }
}

pub(crate) fn eval_chunk(
    chunk: &Chunk,
    scope: &mut LocalScope<Context>,
) -> Result<ControlFlow, EvalError> {
    match chunk {
        Chunk::Statement(stmnt) => eval_stmnt(stmnt, scope),
        Chunk::FnDecl(decl) => eval_fn_decl(decl, scope).map(|_| ControlFlow::Continue),
    }
}
