use crate::lang::{LuaFunction, LuaNumber, LuaValue};

use super::TableRef;

// No nills allowed
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum LuaKey {
    Number(LuaNumber),
    String(String),
    Function(LuaFunction),
    Table(TableRef),
}

impl Eq for LuaKey {}

impl LuaKey {
    pub fn new(value: LuaValue) -> Option<Self> {
        match value {
            LuaValue::Nil => None,
            LuaValue::Number(num) => Some(Self::Number(num)),
            LuaValue::String(str) => Some(Self::String(str)),
            LuaValue::Function(func) => Some(Self::Function(func)),
            LuaValue::Table(table) => Some(Self::Table(table)),
        }
    }
    pub fn number(num: impl Into<LuaNumber>) -> Self {
        Self::Number(num.into())
    }
    pub fn string(str: impl Into<String>) -> Self {
        Self::String(str.into())
    }
}

impl From<LuaKey> for LuaValue {
    fn from(v: LuaKey) -> Self {
        match v {
            LuaKey::Number(num) => Self::Number(num),
            LuaKey::String(str) => Self::String(str),
            LuaKey::Function(func) => Self::Function(func),
            LuaKey::Table(table) => Self::Table(table),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for LuaKey {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use test_util::with_thread_gen;
        match u8::arbitrary(g) % 2 {
            0 => LuaKey::Number(with_thread_gen(LuaNumber::arbitrary)),
            1 => LuaKey::String(with_thread_gen(String::arbitrary)),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            LuaKey::Number(num) => Box::new(num.shrink().map(LuaKey::Number)),
            LuaKey::String(str) => Box::new(str.shrink().map(LuaKey::String)),
            _ => quickcheck::empty_shrinker(),
            // LuaValue::MultiValue(values) => Box::new(values.shrink().map(|values| {
            //     if values.len() == 1 {
            //         values.into_iter().next().unwrap()
            //     } else {
            //         LuaValue::MultiValue(values)
            //     }
            // })),
        }
    }
}
