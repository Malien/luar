use std::{cmp::Ordering, rc::Rc};

use luar_string::{lua_format, LuaString};

use crate::{eq_with_nan::eq_with_nan, ids::BlockID};

use super::{FFIFunc, FromArgs, NativeFunction, TableRef};

#[derive(Debug, Clone)]
pub enum WideLuaValue {
    Nil,
    Int(i32),
    Float(f64),
    String(LuaString),
    NativeFunction(NativeFunction),
    Function(BlockID),
    Table(TableRef),
}

fn is_float_intlike(float: f64) -> bool {
    (float as i32) as f64 == float
}

impl PartialEq for WideLuaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::Int(l0), Self::Float(r0)) if is_float_intlike(*r0) => *l0 == *r0 as i32,
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            (Self::Float(l0), Self::Int(r0)) if is_float_intlike(*l0) => *l0 as i32 == *r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::NativeFunction(l0), Self::NativeFunction(r0)) => l0 == r0,
            (Self::Function(l0), Self::Function(r0)) => l0 == r0,
            (Self::Table(l0), Self::Table(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialOrd for WideLuaValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Ordering::*;

        match (self, other) {
            (Self::Nil, Self::Nil) => Some(Equal),
            (Self::Int(lhs), Self::Int(rhs)) => i32::partial_cmp(lhs, rhs),
            (Self::Int(lhs), Self::Float(rhs)) => f64::partial_cmp(&(*lhs as f64), rhs),
            // TODO: either remove ability to compare numbers and strings,
            //       or provide a version where intermediate string is not being allocated
            (Self::Int(lhs), Self::String(rhs)) => {
                str::partial_cmp(&lhs.to_string(), rhs.as_ref())
            }
            (Self::Float(lhs), Self::Int(rhs)) => f64::partial_cmp(lhs, &(*rhs as f64)),
            (Self::Float(lhs), Self::Float(rhs)) => f64::partial_cmp(lhs, rhs),
            (Self::Float(lhs), Self::String(rhs)) => {
                str::partial_cmp(&lhs.to_string(), &rhs)
            }
            (Self::String(lhs), Self::Int(rhs)) => str::partial_cmp(&lhs, &rhs.to_string()),
            (Self::String(lhs), Self::Float(rhs)) => {
                str::partial_cmp(&lhs, &rhs.to_string())
            }
            (Self::String(lhs), Self::String(rhs)) => str::partial_cmp(lhs, rhs),
            (Self::NativeFunction(lhs), Self::NativeFunction(rhs)) if lhs == rhs => {
                Some(Equal)
            }
            (Self::Function(lhs), Self::Function(rhs)) if lhs == rhs => Some(Equal),
            (Self::Table(lhs), Self::Table(rhs)) if lhs == rhs => Some(Equal),
            _ => None,
        }
    }
}

impl Default for WideLuaValue {
    fn default() -> Self {
        Self::Nil
    }
}

impl WideLuaValue {
    pub fn table(table_ref: TableRef) -> Self {
        Self::Table(table_ref)
    }

    pub fn lua_function(block_id: BlockID) -> Self {
        Self::Function(block_id)
    }

    pub fn native_function(function: NativeFunction) -> Self {
        Self::NativeFunction(function)
    }

    pub fn float(float: f64) -> Self {
        Self::Float(float)
    }

    pub fn string(string: impl Into<LuaString>) -> Self {
        Self::String(string.into())
    }

    pub fn function<'a, F, Args>(func: F) -> Self
    where
        F: FFIFunc<Args> + 'static,
        Args: FromArgs<'a> + 'static,
    {
        Self::NativeFunction(NativeFunction::new(func))
    }

    pub fn int<T>(int: T) -> Self
    where
        T: TryInto<i32>,
        T::Error: std::fmt::Debug,
    {
        Self::Int(int.try_into().unwrap())
    }

    pub fn coerce_to_f64(&self) -> Option<f64> {
        match self {
            Self::Int(int) => Some(*int as f64),
            Self::Float(float) => Some(*float),
            Self::String(str) => str.parse().ok(),
            _ => None,
        }
    }

    pub fn coerce_to_i32(&self) -> Option<i32> {
        match self {
            Self::Int(int) => Some(*int),
            Self::Float(float) => Some(*float as i32),
            Self::String(str) => str.parse().ok(),
            _ => None,
        }
    }

    pub fn coerce_to_usize(&self) -> Option<usize> {
        match self {
            Self::Int(int) => Some(*int as usize),
            Self::Float(float) => Some(*float as usize),
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

    pub fn coerce_to_string(&self) -> Option<LuaString> {
        match self {
            Self::Int(int) => Some(lua_format!("{int}")),
            Self::Float(float) => Some(lua_format!("{float}")),
            Self::String(str) => Some(str.clone()),
            _ => None,
        }
    }

    pub const NIL: Self = Self::Nil;

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub fn is_table(&self) -> bool {
        matches!(self, Self::Table(_))
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

    pub fn unwrap_table(self) -> TableRef {
        if let Self::Table(table_ref) = self {
            return table_ref;
        }
        panic!("Tried to call unwrap_table() on {:?}", self)
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

    pub fn total_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Int(lhs), Self::Int(rhs)) => lhs == rhs,
            (Self::Float(lhs), Self::Float(rhs)) => eq_with_nan(*lhs, *rhs),
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            (Self::NativeFunction(lhs), Self::NativeFunction(rhs)) => lhs == rhs,
            (Self::Function(lhs), Self::Function(rhs)) => lhs == rhs,
            (Self::Table(lhs), Self::Table(rhs)) => lhs == rhs,
            _ => false,
        }
    }

    pub fn is_comparable(&self) -> bool {
        matches!(
            self,
            Self::Int(_) | Self::Float(_) | Self::String(_)
        )
    }
}

impl std::fmt::Display for WideLuaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => f.write_str("nil"),
            Self::Int(int) => std::fmt::Display::fmt(int, f),
            Self::Float(float) => std::fmt::Display::fmt(float, f),
            Self::String(string) => std::fmt::Debug::fmt(string, f),
            Self::NativeFunction(function) => {
                write!(f, "native_function: {:p}", Rc::as_ptr(&function.0))
            }
            Self::Function(block_id) => write!(f, "function: {:#x}", block_id.0),
            Self::Table(table_ref) => write!(f, "table: {:p}", table_ref.as_ptr()),
        }
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for WideLuaValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use test_util::{with_thread_gen, GenExt};

        match u8::arbitrary(g) % 5 {
            0 => Self::Nil,
            1 => Self::Int(with_thread_gen(i32::arbitrary)),
            2 => Self::Float(with_thread_gen(f64::arbitrary)),
            3 => Self::String(with_thread_gen(LuaString::arbitrary)),
            4 => Self::Table(TableRef::arbitrary(&mut g.next_iter())),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::Nil => quickcheck::empty_shrinker(),
            Self::Int(int) => {
                Box::new(std::iter::once(Self::Nil).chain(int.shrink().map(Self::Int)))
            }
            Self::Float(float) => {
                Box::new(std::iter::once(Self::Nil).chain(float.shrink().map(Self::Float)))
            }
            Self::String(str) => {
                Box::new(std::iter::once(Self::Nil).chain(str.shrink().map(Self::String)))
            }
            Self::NativeFunction(_) | Self::Function(_) => {
                Box::new(std::iter::once(Self::Nil))
            }
            Self::Table(table) => {
                Box::new(std::iter::once(Self::Nil).chain(table.shrink().map(Self::Table)))
            }
        }
    }
}

macro_rules! lmatch {
    (
        $value:expr; 
        nil => $nil_match:expr,
        int $int_ident:ident => $int_match:expr,
        float $float_ident:ident => $float_match:expr,
        string $string_ident:ident => $string_match:expr,
        table $table_ident:ident => $table_match:expr,
        native_function $native_function_ident:ident => $native_function_match:expr,
        lua_function $lua_function_ident:ident => $lua_function_match:expr$(,)?
    ) => {{
        match $value {
            $crate::value::wide::WideLuaValue::Nil => $nil_match,
            $crate::value::wide::WideLuaValue::Int($int_ident) => $int_match,
            $crate::value::wide::WideLuaValue::Float($float_ident) => $float_match,
            $crate::value::wide::WideLuaValue::String($string_ident) => $string_match,
            $crate::value::wide::WideLuaValue::Table($table_ident) => $table_match,
            $crate::value::wide::WideLuaValue::NativeFunction($native_function_ident) => {
                $native_function_match
            }
            $crate::value::wide::WideLuaValue::Function($lua_function_ident) => {
                $lua_function_match
            }
        }
    }};

    (
        $value:expr; 
        nil => $nil_match:expr,
        int $int_ident:ident => $int_match:expr,
        float $float_ident:ident => $float_match:expr,
        string ref $string_ident:ident => $string_match:expr,
        table $table_ident:ident => $table_match:expr,
        native_function $native_function_ident:ident => $native_function_match:expr,
        lua_function $lua_function_ident:ident => $lua_function_match:expr$(,)?
    ) => {{
        match $value {
            $crate::value::wide::WideLuaValue::Nil => $nil_match,
            $crate::value::wide::WideLuaValue::Int($int_ident) => $int_match,
            $crate::value::wide::WideLuaValue::Float($float_ident) => $float_match,
            $crate::value::wide::WideLuaValue::String(ref $string_ident) => $string_match,
            $crate::value::wide::WideLuaValue::Table($table_ident) => $table_match,
            $crate::value::wide::WideLuaValue::NativeFunction($native_function_ident) => {
                $native_function_match
            }
            $crate::value::wide::WideLuaValue::Function($lua_function_ident) => {
                $lua_function_match
            }
        }
    }};
}

pub(crate) use lmatch;
