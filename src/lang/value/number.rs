use std::{str::FromStr, hash::Hash};

use crate::util::eq_with_nan;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct LuaNumber(f64);

// Yeah, NaNs but lua spec does not deal with them, so I won't as well
impl Eq for LuaNumber {}

// Same thing with NaNs but lua does not special cases those. It's UB
impl Hash for LuaNumber {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

impl LuaNumber {
    pub fn total_eq(&self, other: &LuaNumber) -> bool {
        eq_with_nan(self.0, other.0)
    }
    pub fn as_f64(self) -> f64 {
        self.0
    }
}

impl FromStr for LuaNumber {
    type Err = <f64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FromStr::from_str(s).map(LuaNumber)
    }
}

impl From<f64> for LuaNumber {
    fn from(v: f64) -> Self {
        LuaNumber(v)
    }
}

impl From<usize> for LuaNumber {
    fn from(v: usize) -> Self {
        LuaNumber(v as f64)
    }
}

impl From<i32> for LuaNumber {
    fn from(v: i32) -> Self {
        LuaNumber(v as f64)
    }
}

impl From<u64> for LuaNumber {
    fn from(v: u64) -> Self {
        LuaNumber(v as f64)
    }
}

impl From<u8> for LuaNumber {
    fn from(v: u8) -> Self {
        LuaNumber(v as f64)
    }
}

impl std::fmt::Display for LuaNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for LuaNumber {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(quickcheck::Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(LuaNumber))
    }
}

// macro_rules! from_num {
//     ($type:ty) => {
//         impl From<$type> for LuaNumber {
//             fn from(v: $type) -> Self {
//                 LuaNumber(v)
//             }
//         }
//     };
// }

// from_num!(f64);
// from_num!(f32);
// from_num!(u64);
// from_num!(u32);
// from_num!(u16);
// from_num!(u8);
// from_num!(i64);
// from_num!(i32);
// from_num!(i16);
// from_num!(i8);
