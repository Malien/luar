use std::error::Error;
use std::fmt;

use super::LuaValue;
use crate::syn;

#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    TypeError(TypeError),
    AssertionError,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    Arithmetic(ArithmeticError),
    IsNotCallable(LuaValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticError {
    UnaryMinus(LuaValue),
    Binary {
        lhs: LuaValue,
        op: syn::BinaryOperator,
        rhs: LuaValue,
    },
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError(err) => fmt::Display::fmt(err, f),
            Self::AssertionError => f.write_str("Assertion failed"),
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
            Self::Binary { lhs, rhs, op } => write!(
                f,
                "Cannot apply operator \"{}\" to operands {} and {}",
                op, lhs, rhs
            ),
        }
    }
}

impl Error for EvalError {}
impl Error for TypeError {}
impl Error for ArithmeticError {}
