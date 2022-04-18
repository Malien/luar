use std::error::Error;
use std::fmt;

use luar_lex::Ident;

use super::{LuaType, LuaValue};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("{0}")]
    TypeError(Box<TypeError>),
    #[error("Assertion failed")]
    AssertionError,
    #[error("IO Error: {0}")]
    IO(std::io::Error),
}

impl From<TypeError> for EvalError {
    fn from(e: TypeError) -> Self {
        Self::TypeError(Box::new(e))
    }
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
    CannotAssignProperty {
        property: Ident,
        of: LuaValue,
    },
    Ordering {
        lhs: LuaValue,
        rhs: LuaValue,
        op: OrderingOperator,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingOperator {
    Less,
    Greater,
    LessOrEquals,
    GreaterOrEquals,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticError {
    UnaryMinus(LuaValue),
    Binary {
        lhs: LuaValue,
        op: ArithmeticOperator,
        rhs: LuaValue,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for OrderingOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Less => "<",
            Self::Greater => ">",
            Self::LessOrEquals => "<=",
            Self::GreaterOrEquals => ">=",
        }
        .fmt(f)
    }
}

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => '+',
            Self::Sub => '-',
            Self::Mul => '*',
            Self::Div => '/',
        }
        .fmt(f)
    }
}

// impl fmt::Display for EvalError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Self::TypeError(err) => fmt::Display::fmt(err, f),
//             Self::AssertionError => f.write_str("Assertion failed"),
//             Self::IO(err) => write!(f, "IO Error: {}", err),
//         }
//     }
// }

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("Type Error: ", f)?;
        match self {
            Self::Arithmetic(err) => write!(f, "Arithmetic Error: {}", err),
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
            Self::CannotAssignProperty { property, of } => {
                write!(f, "Cannot assign to property {} of {}", property, of)
            }
            Self::Ordering { lhs, rhs, op } => {
                write!(
                    f,
                    "Cannot compare {} and {} with an \"{}\" operator",
                    lhs, rhs, op
                )
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

// impl Error for EvalError {}
impl Error for TypeError {}
impl Error for ArithmeticError {}

#[macro_export]
#[cfg(test)]
macro_rules! assert_type_error {
    ($pattern:pat, $value:expr) => {
        if let Err(EvalError::TypeError(err)) = $value {
            if let $pattern = err.as_ref() {
            } else {
                panic!("Unexpected result type");
            }
        } else {
            panic!("Unexpected result type");
        }
    };
}
