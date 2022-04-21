use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

#[derive(Clone, PartialEq, Eq)]
pub struct KeyedVec<K, V> {
    vec: Vec<V>,
    _key: PhantomData<K>,
}

impl<K, V> KeyedVec<K, V> {
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            _key: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            _key: PhantomData,
        }
    }

    pub fn from_vec(vec: Vec<V>) -> Self {
        Self {
            vec,
            _key: PhantomData,
        }
    }

    pub fn values(self) -> std::vec::IntoIter<V> {
        self.vec.into_iter()
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional)
    }

    pub fn slice(&self) -> &[V] {
        &self.vec
    }
}

impl<K, V> KeyedVec<K, V>
where
    K: TryFrom<usize>,
    K::Error: std::fmt::Debug,
{
    pub fn push(&mut self, value: V) -> K {
        let key = self.next_key();
        self.vec.push(value);
        key
    }

    pub fn next_key(&self) -> K {
        self.vec.len().try_into().unwrap()
    }
}

impl<K, V> KeyedVec<K, V>
where
    K: Into<usize>,
    V: Clone,
{
    pub fn accommodate_for_key(&mut self, key: K, value: V) {
        let amount_missing = (key.into() + 1).checked_sub(self.vec.len());
        match amount_missing {
            Some(0) | None => {}
            Some(amount_missing) => {
                self.vec.extend(std::iter::repeat(value).take(amount_missing))
            }
        }
    }
}

impl<K, V> Default for KeyedVec<K, V> {
    fn default() -> Self {
        Self {
            vec: Default::default(),
            _key: Default::default(),
        }
    }
}

impl<K, V> std::fmt::Debug for KeyedVec<K, V> where V: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.vec).finish()
    }
}

impl<K, V> Index<K> for KeyedVec<K, V>
where
    K: Into<usize>,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        &self.vec[index.into()]
    }
}

impl<K, V> IndexMut<K> for KeyedVec<K, V>
where
    K: Into<usize>,
{
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        &mut self.vec[index.into()]
    }
}

macro_rules! keyed_vec {
    ($($expr:expr),*$(,)?) => {
        $crate::keyed_vec::KeyedVec::from_vec(vec![$($expr),*])
    };
}

pub(crate) use keyed_vec;
