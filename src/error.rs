use thiserror::Error;

use crate::lang::EvalError;
use crate::syn::{ParseErrorWithSourcePosition, RawParseError, ParseError};

#[derive(Debug, Error)]
pub enum LuaError {
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Eval(#[from] EvalError),
}

impl From<RawParseError> for LuaError {
    fn from(err: RawParseError) -> Self {
        Self::Parse(ParseError::from(err))
    }
}

impl From<ParseErrorWithSourcePosition> for LuaError {
    fn from(err: ParseErrorWithSourcePosition) -> Self {
        Self::Parse(ParseError::from(err))
    }
}
