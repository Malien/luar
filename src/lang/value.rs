use std::fmt;

use crate::util::eq_with_nan;

#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    Nil,
    Number(f64),
    String(String),
    // Function,
    // Table,
    // CFunction,
    // UserData,
}

impl LuaValue {
    pub fn is_falsy(&self) -> bool {
        matches!(self, LuaValue::Nil)
    }
    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }

    pub fn unwrap_number(self) -> f64 {
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

    pub fn total_eq(&self, other: &LuaValue) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Number(lhs), Self::Number(rhs)) => eq_with_nan(*lhs, *rhs),
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            _ => false
        }
    }
}

impl fmt::Display for LuaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => fmt::Display::fmt("nil", f),
            Self::Number(num) => fmt::Display::fmt(num, f),
            Self::String(str) => fmt::Debug::fmt(str, f),
            // Self::Function => fmt::Display::fmt("<function>", f),
            // Self::Table => fmt::Display::fmt("<table>", f),
            // Self::CFunction => fmt::Display::fmt("<cfunction>", f),
            // Self::UserData => fmt::Display::fmt("<unserdata>", f),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for LuaValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match u8::arbitrary(g) % 3 {
            0 => LuaValue::Nil,
            1 => LuaValue::Number(f64::arbitrary(g)),
            2 => LuaValue::String(String::arbitrary(g)),
            _ => todo!(),
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
        }
    }
}
