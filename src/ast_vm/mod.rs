mod block;
pub(crate) use block::*;

mod expr;
pub(crate) use expr::*;

mod fn_decl;
pub(crate) use fn_decl::*;

mod module;
pub use module::*;

mod ret;
pub(crate) use ret::*;

mod stmnt;
pub(crate) use stmnt::*;

mod var;
pub(crate) use var::*;

mod tail_values;
pub use tail_values::*;

pub mod scope;

mod ctrl_flow;
pub use ctrl_flow::*;

pub use crate::lang::Engine;
use crate::lang::GlobalContext;

pub struct AstVM;

impl Engine for AstVM {
    type ExecutionContext = GlobalContext;

    fn eval_module(
        module: &luar_syn::Module,
        context: &mut Self::ExecutionContext,
    ) -> Result<crate::lang::ReturnValue, crate::lang::EvalError> {
        eval_module(module, context)
    }
}

#[cfg(test)]
pub(crate) fn vec_of_idents(len: usize, prefix: &str) -> Vec<luar_lex::Ident> {
    (0..len)
        .into_iter()
        .map(|i| format!("{}{}", prefix, i))
        .map(luar_lex::Ident::new)
        .collect()
}
