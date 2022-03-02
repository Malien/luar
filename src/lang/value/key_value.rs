use crate::lang::{LuaNumber, LuaFunction, LuaValue};

// No nills allowed
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum LuaKeyValue {
    Number(LuaNumber),
    String(String),
    Function(LuaFunction),
    // Table(TableRef)
}

impl Eq for LuaKeyValue {}

impl LuaKeyValue {
    pub fn new(value: LuaValue) -> Option<Self> {
        match value {
            LuaValue::Nil => None,
            LuaValue::Number(num) => Some(Self::Number(num)),
            LuaValue::String(str) => Some(Self::String(str)),
            LuaValue::Function(func) => Some(Self::Function(func)),
        }
    }
}

impl From<LuaKeyValue> for LuaValue {
    fn from(v: LuaKeyValue) -> Self {
        match v {
            LuaKeyValue::Number(num) => Self::Number(num),
            LuaKeyValue::String(str) => Self::String(str),
            LuaKeyValue::Function(func) => Self::Function(func),
        }
    }
}
