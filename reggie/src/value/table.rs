use crate::{LuaKey, LuaValue};
use std::{borrow::Borrow, cell::{RefCell, RefMut}, collections::HashMap, hash::Hash, ops::Deref, ptr::NonNull, rc::Rc};

use super::LuaString;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableValue {
    array: Vec<LuaValue>,
    hash: HashMap<LuaKey, LuaValue>,
}

fn is_usize_like_float(float: f64) -> bool {
    (float as usize) as f64 == float
}

fn iter_eq_by<T>(
    lhs: impl IntoIterator<Item = T>,
    rhs: impl IntoIterator<Item = T>,
    mut eq: impl FnMut(T, T) -> bool,
) -> bool {
    let mut lhs_iter = lhs.into_iter();
    let mut rhs_iter = rhs.into_iter();

    loop {
        match (lhs_iter.next(), rhs_iter.next()) {
            (Some(lhs), Some(rhs)) => {
                if !eq(lhs, rhs) {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

impl TableValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.array.is_empty() && self.hash.is_empty()
    }

    pub fn get(&self, key: &LuaKey) -> &LuaValue {
        match key {
            LuaKey::Int(int) if *int > 0 && self.array.len() >= *int as usize => {
                &self.array[*int as usize - 1]
            }
            LuaKey::Float(float)
                if *float >= 1.0
                    && is_usize_like_float(float.into_inner())
                    && self.array.len() >= float.into_inner() as usize =>
            {
                &self.array[float.into_inner() as usize - 1]
            }
            key => self.hash.get(key).unwrap_or(&LuaValue::nil_ref()),
        }
    }

    pub fn set(&mut self, key: LuaKey, value: LuaValue) {
        match key {
            LuaKey::Int(int) if int > 0 && self.array.len() >= int as usize => {
                self.array[int as usize - 1] = value;
            }
            LuaKey::Int(int) if int > 0 && self.array.len() + 1 == int as usize => {
                self.array.push(value);
            }
            LuaKey::Float(float)
                if float >= 1.0
                    && is_usize_like_float(float.into_inner())
                    && self.array.len() + 1 >= float.into_inner() as usize =>
            {
                self.array[float.into_inner() as usize - 1] = value;
            }
            LuaKey::Float(float)
                if float >= 1.0
                    && is_usize_like_float(float.into_inner())
                    && self.array.len() == float.into_inner() as usize =>
            {
                self.array.push(value);
            }
            key => {
                self.hash.insert(key, value);
            }
        };
    }

    pub fn total_eq(&self, other: &TableValue) -> bool {
        iter_eq_by(&self.array, &other.array, LuaValue::total_eq)
            && self
                .hash
                .iter()
                .all(|(key, value)| other.get(key).total_eq(value))
            && other
                .hash
                .iter()
                .all(|(key, value)| self.get(key).total_eq(value))
    }

    pub fn push(&mut self, value: LuaValue) {
        self.array.push(value)
    }

    pub fn assoc_str<S: Into<LuaString>>(&mut self, str: S, value: LuaValue) {
        self.hash.insert(LuaKey::String(str.into()), value);
    }

    pub fn get_str_assoc(&mut self, str: impl Into<LuaString>) -> LuaValue {
        self.hash
            .get(&LuaKey::String(str.into()))
            .cloned()
            .unwrap_or_default()
    }
}

#[repr(transparent)]
pub struct UnownedTableRef<'a>(&'a mut RefCell<TableValue>);

impl UnownedTableRef<'_> {
    /// SAFETY: Make sure the lifetime matches the scope
    pub unsafe fn new(mut raw: NonNull<RefCell<TableValue>>) -> Self {
        Self(unsafe { raw.as_mut() })
    }

    pub fn to_owned(&self) -> TableRef {
        TableRef(
            unsafe { Rc::from_raw(self.0 as * const _) }
        )
    }

    pub fn borrow(&self) -> std::cell::Ref<'_, TableValue> {
        RefCell::borrow(self.0)
    }

    pub fn borrow_mut(&mut self) -> RefMut<'_, TableValue> {
        RefCell::borrow_mut(self.0)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct TableRef(pub(crate) Rc<RefCell<TableValue>>);

impl TableRef {
    pub fn as_ptr(&self) -> *const TableValue {
        RefCell::as_ptr(self.0.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        RefCell::borrow(&self.0).is_empty()
    }

    pub fn unwrap_or_clone(self) -> TableValue {
        Rc::try_unwrap(self.0)
            .unwrap_or_else(|rc| (*rc).clone())
            .into_inner()
    }

    pub fn push(&mut self, value: LuaValue) {
        RefCell::borrow_mut(&self.0).push(value)
    }

    pub fn assoc_str<S: Into<LuaString>>(&mut self, str: S, value: LuaValue) {
        RefCell::borrow_mut(&self.0).assoc_str(str, value)
    }

    pub fn get_str_assoc(&mut self, str: impl Into<LuaString>) -> LuaValue {
        self.0.borrow_mut().get_str_assoc(str)
    }

    pub fn get(&self, member: &LuaKey) -> LuaValue {
        self.0.borrow().get(member).clone()
    }

    pub fn set(&mut self, member: LuaKey, value: LuaValue) {
        self.0.borrow_mut().set(member, value)
    }

    /// Construct a new empty table value and reference it
    pub fn new() -> Self {
        Self::from(TableValue::new())
    }
}

impl From<TableValue> for TableRef {
    fn from(value: TableValue) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl Clone for TableRef {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl Hash for TableRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl PartialEq for TableRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TableRef {}

impl Default for TableRef {
    fn default() -> Self {
        Self::from(TableValue::default())
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for TableValue {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self {
            array: quickcheck::Arbitrary::arbitrary(g),
            hash: quickcheck::Arbitrary::arbitrary(g),
        }
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let array = self.array.clone();
        let hash = self.hash.clone();
        Box::new(
            self.array
                .shrink()
                .map(move |array| Self {
                    array,
                    hash: hash.clone(),
                })
                .chain(self.hash.shrink().map(move |hash| Self {
                    array: array.clone(),
                    hash,
                })),
        )
    }
}

#[cfg(feature = "quickcheck")]
impl quickcheck::Arbitrary for TableRef {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Self(Rc::new(RefCell::new(quickcheck::Arbitrary::arbitrary(g))))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            RefCell::borrow(&self.0)
                .shrink()
                .map(|v| Self(Rc::new(RefCell::new(v)))),
        )
    }
}
