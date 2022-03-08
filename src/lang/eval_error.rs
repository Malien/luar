use std::error::Error;
use std::fmt;

use super::{LuaType, LuaValue};
use crate::{lex::Ident, syn};

#[derive(Debug)]
pub enum EvalError {
    TypeError(TypeError),
    AssertionError,
    IO(std::io::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    Arithmetic(ArithmeticError),
    IsNotCallable(LuaValue),
    ArgumentType {
        position: usize,
        expected: LuaType,
        got: LuaType,
    },
    NilLookup,
    IsNotIndexable(LuaValue),
    CannotAccessProperty {
        property: Ident,
        of: LuaValue,
    },
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
            Self::IO(err) => write!(f, "IO Error: {}", err),
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
            Self::ArgumentType {
                position,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Invalid argument type at position {}, expected {}, got {}",
                    position, expected, got
                )
            }
            Self::NilLookup => write!(f, "Tried to perform a table lookup with a nil key"),
            Self::IsNotIndexable(value) => write!(f, "Value {} cannot be indexed", value),
            Self::CannotAccessProperty { property, of } => {
                write!(f, "Cannot access property {} of {}", property, of)
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
