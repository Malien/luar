use luar_syn::{ParseError, ParseErrorWithSourcePosition, RawParseError};
use thiserror::Error;

mod eval_error;
pub use eval_error::*;

#[derive(Debug, Error)]
pub enum LuaError<V> {
    #[error("{0}")]
    Parse(Box<ParseError>),
    #[error("{0}")]
    Eval(#[from] EvalError<V>),
}

impl<V> From<ParseError> for LuaError<V> {
    fn from(err: ParseError) -> Self {
        Self::Parse(Box::new(err))
    }
}

impl<V> From<RawParseError> for LuaError<V> {
    fn from(err: RawParseError) -> Self {
        Self::from(ParseError::from(err))
    }
}

impl<V> From<ParseErrorWithSourcePosition> for LuaError<V> {
    fn from(err: ParseErrorWithSourcePosition) -> Self {
        Self::from(ParseError::from(err))
    }
}
