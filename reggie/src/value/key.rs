use crate::{ids::BlockID, LuaValue, NativeFunction, TableRef};
use decorum::NotNan;
use num_traits::FromPrimitive;

use super::{lmatch, LuaString};

// TODO: CompactLuaKey equivalent
// No nills allowed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LuaKey {
    Int(i32),
    Float(NotNan<f64>),
    String(LuaString),
    NativeFunction(NativeFunction),
    Function(BlockID),
    Table(TableRef),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidLuaKey {
    Nil,
    NaN,
}

impl LuaKey {
    pub fn string(str: impl Into<LuaString>) -> Self {
        Self::String(str.into())
    }
}

impl TryFrom<LuaValue> for LuaKey {
    type Error = InvalidLuaKey;

    fn try_from(value: LuaValue) -> Result<Self, Self::Error> {
        // TODO: Make a lmatch! { move value; ... } macro to avoid inner-ref-counting
        lmatch! { value;
            nil => Err(InvalidLuaKey::Nil),
            int int => Ok(Self::Int(int)),
            float float => NotNan::from_f64(float)
                .map(Self::Float)
                .ok_or(InvalidLuaKey::NaN),
            string str => Ok(Self::String(str)),
            table table => Ok(Self::Table(table)),
            native_function func => Ok(Self::NativeFunction(func)),
            lua_function func => Ok(Self::Function(func)),
        }
    }
}

impl From<LuaKey> for LuaValue {
    fn from(v: LuaKey) -> Self {
        match v {
            LuaKey::Int(int) => Self::int(int),
            LuaKey::Float(float) => Self::float(float.into()),
            LuaKey::String(str) => Self::string(str),
            LuaKey::NativeFunction(func) => Self::native_function(func),
            LuaKey::Function(func) => Self::lua_function(func),
            LuaKey::Table(table) => Self::table(table),
        }
    }
}

#[cfg(feature = "quickcheck")]
fn arbitrary_non_nan_f64(g: &mut quickcheck::Gen) -> NotNan<f64> {
    loop {
        let res = quickcheck::Arbitrary::arbitrary(g);
        if let Some(res) = NotNan::from_f64(res) {
            return res;
        }
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for LuaKey {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use test_util::{with_thread_gen, GenExt};

        match u8::arbitrary(g) % 4 {
            0 => LuaKey::Int(with_thread_gen(i32::arbitrary)),
            1 => LuaKey::Float(with_thread_gen(arbitrary_non_nan_f64)),
            2 => LuaKey::String(with_thread_gen(LuaString::arbitrary)),
            3 => LuaKey::Table(TableRef::arbitrary(&mut g.next_iter())),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            LuaKey::Int(int) => Box::new(int.shrink().map(LuaKey::Int)),
            LuaKey::Float(float) => Box::new(
                float
                    .into_inner()
                    .shrink()
                    .filter_map(NotNan::from_f64)
                    .map(LuaKey::Float),
            ),
            LuaKey::String(str) => Box::new(str.shrink().map(LuaKey::String)),
            _ => quickcheck::empty_shrinker(),
        }
    }
}
