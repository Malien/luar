use luar_syn::{ParseError, ParseErrorWithSourcePosition, RawParseError};
use thiserror::Error;

mod eval_error;
pub use eval_error::*;

#[derive(Debug, Error)]
pub enum LuaError<Value, Str> {
    #[error("{0}")]
    Parse(Box<ParseError>),
    #[error("{0}")]
    Eval(#[from] EvalError<Value, Str>),
}

impl<Value, Str> From<ParseError> for LuaError<Value, Str> {
    fn from(err: ParseError) -> Self {
        Self::Parse(Box::new(err))
    }
}

impl<Value, Str> From<RawParseError> for LuaError<Value, Str> {
    fn from(err: RawParseError) -> Self {
        Self::from(ParseError::from(err))
    }
}

impl<Value, Str> From<ParseErrorWithSourcePosition> for LuaError<Value, Str> {
    fn from(err: ParseErrorWithSourcePosition) -> Self {
        Self::from(ParseError::from(err))
    }
}
