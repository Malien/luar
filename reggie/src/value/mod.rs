use std::rc::Rc;

use crate::{eq_with_nan::eq_with_nan, ids::BlockID};

pub mod native_function;
pub use native_function::*;

pub mod traits;
pub use traits::*;

pub mod signature;

#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    Nil,
    Int(i32),
    Float(f64),
    String(String),
    NativeFunction(NativeFunction),
    Function(BlockID),
}

impl Default for LuaValue {
    fn default() -> Self {
        Self::Nil
    }
}

impl LuaValue {
    pub fn string(string: impl Into<String>) -> Self {
        Self::String(string.into())
    }

    pub fn coerce_to_f64(&self) -> Option<f64> {
        match self {
            Self::Int(int) => Some(*int as f64),
            Self::Float(float) => Some(*float),
            Self::String(str) => str.parse().ok(),
            _ => None,
        }
    }

    pub fn number_as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(int) => Some(*int as f64),
            Self::Float(float) => Some(*float),
            _ => None,
        }
    }

    pub fn is_table(&self) -> bool {
        // matches!(self, Self::Table(_))
        false
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Self::NativeFunction(_) | Self::Function(_))
    }

    pub fn is_truthy(&self) -> bool {
        !self.is_falsy()
    }

    pub fn is_falsy(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn unwrap_int(&self) -> i32 {
        if let Self::Int(int) = self {
            return *int;
        }
        panic!("Tried to call unwrap_int() on {:?}", self)
    }

    pub fn unwrap_lua_function(&self) -> BlockID {
        if let Self::Function(block_id) = self {
            return *block_id;
        }
        panic!("Tried to call unwrap_lua_function() on {:?}", self)
    }

    pub fn as_lua_function(self) -> Option<BlockID> {
        if let Self::Function(block_id) = self {
            Some(block_id)
        } else {
            None
        }
    }

    pub fn true_value() -> Self {
        Self::Int(1)
    }

    pub fn false_value() -> Self {
        Self::Nil
    }

    pub fn from_bool(v: bool) -> Self {
        if v {
            Self::true_value()
        } else {
            Self::false_value()
        }
    }

    pub fn total_eq(&self, other: &LuaValue) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Int(lhs), Self::Int(rhs)) => lhs == rhs,
            (Self::Float(lhs), Self::Float(rhs)) => eq_with_nan(*lhs, *rhs),
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl std::fmt::Display for LuaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaValue::Nil => f.write_str("nil"),
            LuaValue::Int(int) => std::fmt::Display::fmt(int, f),
            LuaValue::Float(float) => std::fmt::Display::fmt(float, f),
            LuaValue::String(string) => std::fmt::Debug::fmt(string, f),
            LuaValue::NativeFunction(function) => {
                write!(f, "native_function: {:p}", Rc::as_ptr(&function.0))
            }
            LuaValue::Function(block_id) => write!(f, "function: {:#x}", block_id.0),
        }
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for LuaValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use test_util::with_thread_gen;

        match u8::arbitrary(g) % 4 {
            0 => LuaValue::Nil,
            1 => LuaValue::Int(with_thread_gen(i32::arbitrary)),
            2 => LuaValue::Float(with_thread_gen(f64::arbitrary)),
            3 => LuaValue::String(with_thread_gen(String::arbitrary)),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            LuaValue::Nil => quickcheck::empty_shrinker(),
            LuaValue::Int(int) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(int.shrink().map(LuaValue::Int)))
            }
            LuaValue::Float(float) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(float.shrink().map(LuaValue::Float)))
            }
            LuaValue::String(str) => {
                Box::new(std::iter::once(LuaValue::Nil).chain(str.shrink().map(LuaValue::String)))
            }
            LuaValue::NativeFunction(_) | LuaValue::Function(_) => {
                Box::new(std::iter::once(LuaValue::Nil))
            }
        }
    }
}
