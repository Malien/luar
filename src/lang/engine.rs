use crate::syn;

use super::{ReturnValue, EvalError};

pub trait Engine {
    type ExecutionContext;

    fn eval_module(
        module: &syn::Module,
        context: &mut Self::ExecutionContext,
    ) -> Result<ReturnValue, EvalError>;
}
