use crate::syn::Var;

use super::{EvalContext, EvalContextExt, LuaValue};

mod block;
mod expr;
mod fn_call;
mod fn_decl;
mod module;
mod ret;
mod stmnt;
mod table;
mod var;

mod tail_values;
pub use tail_values::*;

pub fn assign_to_var<Context>(context: &mut Context, var: &Var, value: LuaValue)
where
    Context: EvalContext + ?Sized,
{
    match var {
        Var::Named(ident) => context.set(ident.clone(), value),
        _ => todo!(),
    }
}
