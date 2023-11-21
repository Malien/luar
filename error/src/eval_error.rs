use std::error::Error;
use std::fmt;

use luar_lex::Ident;

// use super::{LuaType, LuaValue};

#[derive(Debug, thiserror::Error)]
pub enum EvalError<V> {
    TypeError(Box<TypeError<V>>),
    AssertionError(Option<String>),
    IO(std::io::Error),
    Utf8Error,
}

impl<V: fmt::Display> fmt::Display for EvalError<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError(err) => err.fmt(f),
            Self::AssertionError(Some(msg)) => write!(f, "Assertion failed: {}", msg),
            Self::AssertionError(None) => write!(f, "Assertion failed"),
            Self::IO(err) => write!(f, "IO Error: {}", err),
            Self::Utf8Error => write!(f, "Operation produced invalid utf-8 sequence"),
        }
    }
}

impl<V> From<TypeError<V>> for EvalError<V> {
    fn from(e: TypeError<V>) -> Self {
        Self::TypeError(Box::new(e))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError<V> {
    Arithmetic(ArithmeticError<V>),
    IsNotCallable(V),
    ArgumentType {
        position: usize,
        expected: ExpectedType,
        got: V,
    },
    NilAssign(V),
    NaNAssign(V),
    IsNotIndexable(V),
    CannotAccessProperty {
        property: Ident,
        of: V,
    },
    CannotAssignProperty {
        property: Ident,
        of: V,
    },
    CannotAccessMember {
        member: V,
        of: V,
    },
    CannotAssignMember {
        member: V,
        of: V,
    },
    Ordering {
        lhs: V,
        rhs: V,
        op: Option<OrderingOperator>,
    },
    StringConcat {
        lhs: V,
        rhs: V,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedType {
    Number,
    String
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingOperator {
    Less,
    Greater,
    LessOrEquals,
    GreaterOrEquals,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticError<V> {
    UnaryMinus(V),
    Binary {
        lhs: V,
        op: ArithmeticOperator,
        rhs: V,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOperator {
    Add,
    Sub,
    Mul,
    Div,
}

impl fmt::Display for ExpectedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpectedType::Number => "number",
            ExpectedType::String => "string",
        }
        .fmt(f)
    }
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

impl<V: fmt::Display> fmt::Display for TypeError<V> {
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
            Self::NilAssign(value) => {
                write!(f, "Tried to assign value {} to a nil key in a table", value)
            }
            Self::NaNAssign(value) => {
                write!(f, "Tried to assign value {} to a NaN key in a table", value)
            }
            Self::IsNotIndexable(value) => write!(f, "Value {} cannot be indexed", value),
            Self::CannotAccessProperty { property, of } => {
                write!(f, "Cannot access property {} of {}", property, of)
            }
            Self::CannotAssignProperty { property, of } => {
                write!(f, "Cannot assign to property {} of {}", property, of)
            }
            Self::CannotAccessMember { member, of } => {
                write!(f, "Cannot access member {} of {}", member, of)
            }
            Self::CannotAssignMember { member, of } => {
                write!(f, "Cannot assign to a member {} of {}", member, of)
            }
            Self::Ordering {
                lhs,
                rhs,
                op: Some(op),
            } => {
                write!(
                    f,
                    "Cannot compare {} and {} with an \"{}\" operator",
                    lhs, rhs, op
                )
            }
            Self::Ordering { lhs, rhs, op: None } => {
                write!(f, "Cannot compare {} and {}", lhs, rhs)
            }
            Self::StringConcat { lhs, rhs } => {
                write!(f, "Cannot do a string concatenation of {} and {}", lhs, rhs)
            }
        }
    }
}

impl<V: fmt::Display> fmt::Display for ArithmeticError<V> {
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
impl<V: fmt::Display + fmt::Debug> Error for TypeError<V> {}
impl<V: fmt::Display + fmt::Debug> Error for ArithmeticError<V> {}

#[macro_export]
macro_rules! assert_type_error {
    ($pattern:pat, $value:expr) => {
        if let ::std::result::Result::Err($crate::EvalError::TypeError(err)) = $value {
            if let $pattern = err.as_ref() {
            } else {
                panic!("Unexpected result type");
            }
        } else {
            panic!("Unexpected result type");
        }
    };
}
