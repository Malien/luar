use std::error::Error;
use std::fmt;

use super::LuaValue;

#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    TypeError(TypeError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    Arithmetic(ArithmeticError),
    IsNotCallable(LuaValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticError {
    UnaryMinus(LuaValue),
    // Binary {
    //     lhs: LuaValue,
    //     op: syn::BinaryOperator,
    //     rhs: LuaValue,
    // },
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("Type Error: ", f)?;
        match self {
            Self::Arithmetic(err) => fmt::Display::fmt(err, f),
            Self::IsNotCallable(value) => {
                write!(f, "Attempting to call {}, which is not callable", value)
            }
        }
    }
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnaryMinus(value) => write!(f, "Cannot apply unary minus operator to {}", value),
        }
    }
}

impl Error for EvalError {}
impl Error for TypeError {}
impl Error for ArithmeticError {}
