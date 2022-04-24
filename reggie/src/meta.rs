use std::num::NonZeroU16;

use crate::{keyed_vec::KeyedVec, ids::{StringID, JmpLabel}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LocalRegCount {
    pub f: u16,
    pub i: u16,
    pub s: u16,
    pub t: u16,
    pub c: u16,
    pub d: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnID(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArgumentCount {
    Known(u16),
    Unknown,
}

impl Default for ArgumentCount {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<u16> for ArgumentCount {
    fn from(v: u16) -> Self {
        Self::Known(v)
    }
}

impl std::fmt::Display for ArgumentCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Known(count) => {
                write!(f, "(")?;
                if count > 0 {
                    write!(f, "D")?;
                }
                for _ in 1..count {
                    write!(f, ", D")?;
                }
                write!(f, ")")?;
            },
            Self::Unknown => write!(f, "(?)")?
        };
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnCount {
    Unbounded,
    MinBounded(NonZeroU16),
    Bounded { min: u16, max: NonZeroU16 },
    Constant(u16),
}

impl Default for ReturnCount {
    fn default() -> Self {
        Self::Unbounded
    }
}

impl From<u16> for ReturnCount {
    fn from(value: u16) -> Self {
        Self::Constant(value)
    }
}

impl std::fmt::Display for ReturnCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ReturnCount::Unbounded => write!(f, "(?)")?,
            ReturnCount::MinBounded(min) => {
                write!(f, "(")?;
                write!(f, "D")?;
                for _ in 1..min.get() {
                    write!(f, ", D")?;
                }
                write!(f, ", ?)")?;
            },
            ReturnCount::Bounded { min, max } => {
                write!(f, "(")?;
                if min > 0 {
                    write!(f, "D")?;
                }
                for _ in 1..min {
                    write!(f, ", D")?;
                }
                if min == 0 {
                    write!(f, "D?")?;
                }
                for _ in 1..max.get() {
                    write!(f, ", D?")?;
                }
                write!(f, ")")?;
            },
            ReturnCount::Constant(count) => {
                write!(f, "(")?;
                if count > 0 {
                    write!(f, "D")?;
                }
                for _ in 1..count {
                    write!(f, ", D")?;
                }
                write!(f, ")")?;
            },
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CodeMeta {
    // pub identity: FnID,
    // pub source: syn::FunctionDeclaration,
    pub arg_count: ArgumentCount,
    pub local_count: LocalRegCount,
    pub return_count: ReturnCount,
    pub label_mappings: KeyedVec<JmpLabel, u32>,
    pub const_strings: KeyedVec<StringID, String>,
    // pub global_deps:
}
