use thiserror::Error;

use crate::lang::EvalError;
use crate::syn::{ParseError, ParseErrorWithSourcePosition, RawParseError};

#[derive(Debug, Error)]
pub enum LuaError {
    #[error("{0}")]
    Parse(Box<ParseError>),
    #[error("{0}")]
    Eval(#[from] EvalError),
}

impl From<ParseError> for LuaError {
    fn from(err: ParseError) -> Self {
        Self::Parse(Box::new(err))
    }
}

impl From<RawParseError> for LuaError {
    fn from(err: RawParseError) -> Self {
        Self::from(ParseError::from(err))
    }
}

impl From<ParseErrorWithSourcePosition> for LuaError {
    fn from(err: ParseErrorWithSourcePosition) -> Self {
        Self::from(ParseError::from(err))
    }
}
