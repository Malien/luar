use std::{
    iter::Enumerate,
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

    pub fn into_values(self) -> std::vec::IntoIter<V> {
        self.vec.into_iter()
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            vec: self.vec.iter().enumerate(),
            _key: PhantomData,
        }
    }

    pub fn keys(&self) -> KeysIter<K> {
        KeysIter {
            current: 0,
            max: self.len(),
            _key: PhantomData,
        }
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

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
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

    pub fn key_range(&self) -> std::ops::Range<K> {
        let start = 0.try_into().unwrap();
        let end = self.vec.len().try_into().unwrap();
        start..end
    }
}

impl<K, V> KeyedVec<K, V>
where
    K: Into<usize>,
{
    pub fn get(&self, key: K) -> Option<&V> {
        self.vec.get(key.into())
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.vec.get_mut(key.into())
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
            Some(amount_missing) => self
                .vec
                .extend(std::iter::repeat(value).take(amount_missing)),
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

impl<K, V> std::fmt::Debug for KeyedVec<K, V>
where
    V: std::fmt::Debug,
{
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

pub struct Iter<'a, K, V> {
    vec: Enumerate<std::slice::Iter<'a, V>>,
    _key: PhantomData<K>,
}

impl<'a, K, V> IntoIterator for &'a KeyedVec<K, V>
where
    K: TryFrom<usize>,
    K::Error: std::fmt::Debug,
{
    type Item = (K, &'a V);

    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            vec: self.vec.iter().enumerate(),
            _key: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: TryFrom<usize>,
    K::Error: std::fmt::Debug,
{
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.vec
            .next()
            .map(|(idx, value)| (idx.try_into().unwrap(), value))
    }
}

pub struct IterMut<'a, K, V> {
    vec: Enumerate<std::slice::IterMut<'a, V>>,
    _key: PhantomData<K>,
}

impl<'a, K, V> IntoIterator for &'a mut KeyedVec<K, V>
where
    K: TryFrom<usize>,
    K::Error: std::fmt::Debug,
{
    type Item = (K, &'a mut V);

    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            vec: self.vec.iter_mut().enumerate(),
            _key: PhantomData,
        }
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
where
    K: TryFrom<usize>,
    K::Error: std::fmt::Debug,
{
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.vec
            .next()
            .map(|(idx, value)| (idx.try_into().unwrap(), value))
    }
}

pub struct IntoIter<K, V> {
    vec: Enumerate<std::vec::IntoIter<V>>,
    _key: PhantomData<K>,
}

impl<K, V> IntoIterator for KeyedVec<K, V>
where
    K: TryFrom<usize>,
    K::Error: std::fmt::Debug,
{
    type Item = (K, V);

    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            vec: self.vec.into_iter().enumerate(),
            _key: PhantomData,
        }
    }
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: TryFrom<usize>,
    K::Error: std::fmt::Debug,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.vec
            .next()
            .map(|(idx, value)| (idx.try_into().unwrap(), value))
    }
}

pub struct KeysIter<K> {
    current: usize,
    max: usize,
    _key: PhantomData<K>,
}

impl<K> Iterator for KeysIter<K>
where
    K: TryFrom<usize>,
    K::Error: std::fmt::Debug,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.max {
            None
        } else {
            self.current += 1;
            Some(self.current.try_into().unwrap())
        }
    }
}

#[macro_export]
macro_rules! keyed_vec {
    ($($expr:expr),*$(,)?) => {
        $crate::KeyedVec::from_vec(vec![$($expr),*])
    };
}
