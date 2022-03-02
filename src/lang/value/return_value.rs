use std::fmt::{self, Write};

use crate::{
    lang::{LuaFunction, LuaNumber},
    util::NonEmptyVec,
};

use super::LuaValue;

#[derive(Debug, Clone, PartialEq)]
pub enum ReturnValue {
    Nil,
    Number(LuaNumber),
    String(String),
    Function(LuaFunction),
    MultiValue(NonEmptyVec<LuaValue>),
}

impl From<LuaValue> for ReturnValue {
    fn from(v: LuaValue) -> Self {
        match v {
            LuaValue::Nil => Self::Nil,
            LuaValue::Number(num) => Self::Number(num),
            LuaValue::String(str) => Self::String(str),
            LuaValue::Function(func) => Self::Function(func),
        }
    }
}

impl ReturnValue {
    pub fn first_value(self) -> LuaValue {
        match self {
            ReturnValue::Nil => LuaValue::Nil,
            ReturnValue::Number(num) => LuaValue::Number(num),
            ReturnValue::String(str) => LuaValue::String(str),
            ReturnValue::Function(func) => LuaValue::Function(func),
            ReturnValue::MultiValue(values) => values.move_first(),
        }
    }

    pub fn assert_single(self) -> LuaValue {
        match self {
            ReturnValue::Nil => LuaValue::Nil,
            ReturnValue::Number(num) => LuaValue::Number(num),
            ReturnValue::String(str) => LuaValue::String(str),
            ReturnValue::Function(func) => LuaValue::Function(func),
            ReturnValue::MultiValue(values) => {
                assert_eq!(values.len(), 1);
                values.move_first()
            }
        }
    }

    pub fn total_eq(&self, other: &ReturnValue) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Number(lhs), Self::Number(rhs)) => lhs.total_eq(rhs),
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            (Self::Function(lhs), Self::Function(rhs)) => lhs == rhs,
            (Self::MultiValue(lhs), Self::MultiValue(rhs)) if lhs.len() == rhs.len() => {
                lhs.into_iter().zip(rhs).all(|(lhs, rhs)| lhs.total_eq(rhs))
            }
            (_, _) => false,
        }
    }

    pub fn number(value: impl Into<LuaNumber>) -> Self {
        Self::Number(value.into())
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }
}

impl fmt::Display for ReturnValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => fmt::Display::fmt("nil", f),
            Self::Number(num) => fmt::Display::fmt(num, f),
            Self::String(str) => fmt::Debug::fmt(str, f),
            Self::Function(function) => fmt::Debug::fmt(function, f),
            Self::MultiValue(values) => {
                for value in values {
                    fmt::Display::fmt(value, f)?;
                    f.write_char('\t')?;
                }
                Ok(())
            }
            // Self::Table => fmt::Display::fmt("<table>", f),
            // Self::CFunction => fmt::Display::fmt("<cfunction>", f),
            // Self::UserData => fmt::Display::fmt("<unserdata>", f),
        }
    }
}
