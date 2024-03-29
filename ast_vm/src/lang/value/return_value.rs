use luar_string::LuaString;
use smallvec::{smallvec, smallvec_inline, SmallVec};
use std::fmt::{self, Write};

use crate::lang::LuaNumber;

use super::LuaValue;

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnValue(pub SmallVec<[LuaValue; 1]>);

impl From<LuaValue> for ReturnValue {
    fn from(v: LuaValue) -> Self {
        Self(smallvec![v])
    }
}

impl ReturnValue {
    pub const NIL: ReturnValue = ReturnValue(smallvec_inline![LuaValue::Nil]);

    pub fn first_value(self) -> LuaValue {
        self.0.into_iter().next().expect("ReturnValue should have at least one value (even if it is nil)")
    }

    pub fn assert_single(self) -> LuaValue {
        assert!(self.0.len() <= 1);
        self.first_value()
    }

    pub fn total_eq(&self, other: &ReturnValue) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(other.0.iter())
                .all(|(lhs, rhs)| lhs.total_eq(rhs))
    }

    pub fn number(value: impl Into<LuaNumber>) -> Self {
        Self::from(LuaValue::number(value))
    }

    pub fn string(value: impl Into<LuaString>) -> Self {
        Self::from(LuaValue::string(value))
    }

    pub fn true_value() -> Self {
        Self::number(1i32)
    }

    pub fn false_value() -> Self {
        Self::from(LuaValue::Nil)
    }

    pub fn is_multiple_return(&self) -> bool {
        self.0.len() > 1
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Display for ReturnValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for value in self.0.iter() {
            fmt::Display::fmt(value, f)?;
            f.write_char('\t')?;
        }
        Ok(())
    }
}

impl IntoIterator for ReturnValue {
    type Item = LuaValue;
    type IntoIter = <SmallVec<[LuaValue; 1]> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<LuaValue> for ReturnValue {
    fn from_iter<T: IntoIterator<Item = LuaValue>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl std::ops::Index<usize> for ReturnValue {
    type Output = LuaValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for ReturnValue {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
