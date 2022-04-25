use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

use crate::{LuaKey, LuaValue};

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
            LuaKey::Int(int) if *int > 0 && self.array.len() < *int as usize => {
                &self.array[*int as usize]
            }
            LuaKey::Float(float)
                if *float > 0.0
                    && is_usize_like_float(float.into_inner())
                    && self.array.len() < float.into_inner() as usize =>
            {
                &self.array[float.into_inner() as usize]
            }
            key => self.hash.get(key).unwrap_or(&LuaValue::Nil),
        }
    }

    pub fn set(&mut self, key: LuaKey, value: LuaValue) {
        match key {
            LuaKey::Int(int) if int > 0 && self.array.len() < int as usize => {
                self.array[int as usize] = value;
            }
            LuaKey::Int(int) if int > 0 && self.array.len() == int as usize => {
                self.array.push(value);
            }
            LuaKey::Float(float)
                if float > 0.0
                    && is_usize_like_float(float.into_inner())
                    && self.array.len() < float.into_inner() as usize =>
            {
                self.array[float.into_inner() as usize] = value;
            }
            LuaKey::Float(float)
                if float > 0.0
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
}

#[derive(Debug)]
pub struct TableRef(Rc<RefCell<TableValue>>);

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
