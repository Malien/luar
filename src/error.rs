use thiserror::Error;

use crate::lang::EvalError;
use crate::syn::ParseError;

#[derive(Debug, Error)]
pub enum LuaError {
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Eval(#[from] EvalError),
}
