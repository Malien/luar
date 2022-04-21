use std::fmt;

use crate::lang::{EvalError, GlobalContext, LuaFunction, LuaNumber, LuaType};

#[cfg(test)]
use test_util::{with_thread_gen, GenExt};

use super::{ReturnValue, TableRef, TableValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuaValue {
    Nil,
    Number(LuaNumber),
    String(String),
    Function(LuaFunction),
    Table(TableRef),
    // CFunction,
    // UserData
}

impl Default for LuaValue {
    fn default() -> Self {
        Self::Nil
    }
}

impl LuaValue {
    pub fn is_falsy(&self) -> bool {
        matches!(self, LuaValue::Nil)
    }
    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }

    pub fn number(value: impl Into<LuaNumber>) -> LuaValue {
        Self::Number(value.into())
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn table(value: TableValue) -> Self {
        Self::Table(TableRef::from(value))
    }

    pub fn function(
        func: impl Fn(&mut GlobalContext, &[LuaValue]) -> Result<ReturnValue, EvalError> + 'static,
    ) -> Self {
        Self::Function(LuaFunction::new(func))
    }

    pub fn unwrap_number(self) -> LuaNumber {
        if let Self::Number(num) = self {
            return num;
        }
        panic!("Called unwrap_number() on a {:?}", self)
    }

    pub fn unwrap_string(self) -> String {
        if let Self::String(str) = self {
            return str;
        }
        panic!("Called unwrap_string() on a {:?}", self)
    }

    pub fn unwrap_table(self) -> TableRef {
        if let Self::Table(table) = self {
            return table;
        }
        panic!("Called unwrap_table() on a {:?}", self)
    }

    pub fn unwrap_function(self) -> LuaFunction {
        if let Self::Function(function) = self {
            return function;
        }
        panic!("Called unwrap_function() on a {:?}", self)
    }

    pub fn unwrap_function_ref(&self) -> &LuaFunction {
        if let Self::Function(function) = self {
            return function;
        }
        panic!("Called unwrap_function() on a {:?}", self)
    }

    pub fn total_eq(&self, other: &LuaValue) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Number(lhs), Self::Number(rhs)) => lhs.total_eq(rhs),
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            (Self::Function(lhs), Self::Function(rhs)) => lhs == rhs,
            (Self::Table(lhs), Self::Table(rhs)) => lhs == rhs,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    pub fn is_table(&self) -> bool {
        matches!(self, Self::Table(_))
    }

    pub fn as_number(&self) -> Option<LuaNumber> {
        match self {
            LuaValue::Number(num) => Some(*num),
            LuaValue::String(str) => str.parse().ok(),
            _ => None,
        }
    }

    pub fn as_function(self) -> Option<LuaFunction> {
        match self {
            LuaValue::Function(func) => Some(func),
            _ => None,
        }
    }

    pub fn as_function_ref(&self) -> Option<&LuaFunction> {
        match self {
            LuaValue::Function(func) => Some(&func),
            _ => None,
        }
    }

    pub fn true_value() -> LuaValue {
        LuaValue::number(1)
    }

    pub fn false_value() -> LuaValue {
        LuaValue::Nil
    }

    pub fn from_bool(cond: bool) -> LuaValue {
        if cond {
            LuaValue::true_value()
        } else {
            LuaValue::false_value()
        }
    }

    pub fn type_of(&self) -> LuaType {
        match self {
            Self::Nil => LuaType::Nil,
            Self::Number(_) => LuaType::Number,
            Self::String(_) => LuaType::String,
            Self::Function(_) => LuaType::Function,
            Self::Table(_) => LuaType::Table,
        }
    }

    pub fn is_comparable(&self) -> bool {
        self.type_of().is_comparable()
    }
}

impl fmt::Display for LuaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => fmt::Display::fmt("nil", f),
            Self::Number(num) => fmt::Display::fmt(num, f),
            Self::String(str) => fmt::Debug::fmt(str, f),
            Self::Function(function) => fmt::Debug::fmt(function, f),
            Self::Table(table) => fmt::Debug::fmt(table, f),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for LuaValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match u8::arbitrary(g) % 5 {
            0 => LuaValue::Nil,
            1 => LuaValue::Number(with_thread_gen(LuaNumber::arbitrary)),
            2 => LuaValue::String(with_thread_gen(String::arbitrary)),
            3 => LuaValue::Function(LuaFunction::new(|_, _| Ok(ReturnValue::Nil))),
            4 => LuaValue::Table(TableRef::arbitrary(&mut g.next_iter())),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            LuaValue::Nil => quickcheck::empty_shrinker(),
            LuaValue::Number(num) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(num.shrink().map(LuaValue::Number)))
            }
            LuaValue::String(str) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(str.shrink().map(LuaValue::String)))
            }
            LuaValue::Function(_) => Box::new(std::iter::once(LuaValue::Nil)),
            LuaValue::Table(table) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(table.shrink().map(LuaValue::Table)))
            }
        }
    }
}