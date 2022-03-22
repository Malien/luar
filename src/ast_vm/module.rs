use crate::{
    lang::{EvalError, GlobalContext, LocalScope, ReturnValue, ScopeHolder},
    syn::{Chunk, Module},
};

use super::{eval_fn_decl, eval_ret, eval_stmnt, ControlFlow};

pub fn eval_module(module: &Module, context: &mut GlobalContext) -> Result<ReturnValue, EvalError> {
    let mut scope = context.top_level_scope();
    for chunk in &*module.chunks {
        if let ControlFlow::Return(value) = eval_chunk(chunk, &mut scope)? {
            return Ok(value);
        }
    }
    match module.ret {
        Some(ref ret) => eval_ret(ret, &mut scope),
        None => Ok(ReturnValue::Nil),
    }
}

pub(crate) fn eval_chunk(
    chunk: &Chunk,
    scope: &mut LocalScope<GlobalContext>,
) -> Result<ControlFlow, EvalError> {
    match chunk {
        Chunk::Statement(stmnt) => eval_stmnt(stmnt, scope),
        Chunk::FnDecl(decl) => eval_fn_decl(decl, scope).map(|_| ControlFlow::Continue),
    }
}
