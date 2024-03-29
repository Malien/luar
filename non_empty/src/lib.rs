use std::{
    fmt,
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyVec<T>(Vec<T>);

pub struct VecIsEmptyError<T>(pub Vec<T>);

impl<T> fmt::Display for VecIsEmptyError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Tried to construct NonEmptyVec from a empty Vec")
    }
}

impl<T> fmt::Debug for VecIsEmptyError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VecIsEmptyError")
    }
}

impl<T> std::error::Error for VecIsEmptyError<T> {}

impl<T> NonEmptyVec<T> {
    pub fn of_single(value: T) -> Self {
        Self(vec![value])
    }

    pub fn with_head(value: T, mut tail: Vec<T>) -> Self {
        tail.insert(0, value);
        Self(tail)
    }

    pub fn try_new(vec: Vec<T>) -> Result<Self, VecIsEmptyError<T>> {
        if vec.is_empty() {
            Err(VecIsEmptyError(vec))
        } else {
            Ok(NonEmptyVec(vec))
        }
    }

    pub fn from_iter<A: IntoIterator<Item = T>>(iter: A) -> Result<Self, VecIsEmptyError<T>> {
        let vec: Vec<_> = iter.into_iter().collect();
        Self::try_new(vec)
    }

    pub unsafe fn from_iter_unchecked<A: IntoIterator<Item = T>>(iter: A) -> Self {
        Self(iter.into_iter().collect())
    }

    // Panics if vec is empty
    pub fn new(vec: Vec<T>) -> Self {
        assert!(vec.len() != 0);
        Self(vec)
    }

    pub unsafe fn new_unchecked(vec: Vec<T>) -> Self {
        NonEmptyVec(vec)
    }

    pub fn new_with_tail(mut vec: Vec<T>, tail: T) -> Self {
        vec.push(tail);
        Self(vec)
    }

    pub fn first(&self) -> &'_ T {
        // Can be unwrap_unchecked() but I'm scared tbh.
        self.0.first().unwrap()
    }

    pub fn first_mut(&mut self) -> &mut T {
        // Can be unwrap_unchecked() but I'm scared tbh.
        self.0.first_mut().unwrap()
    }

    pub fn last(&self) -> &'_ T {
        // Can be unwrap_unchecked() but I'm scared tbh.
        self.0.last().unwrap()
    }

    pub fn last_mut(&mut self) -> &mut T {
        // Can be unwrap_unchecked() but I'm scared tbh.
        self.0.last_mut().unwrap()
    }

    pub fn unwrap(self) -> Vec<T> {
        self.0
    }

    pub fn move_first(self) -> T {
        // Can be unwrap_unchecked() but I'm scared tbh.
        self.into_iter().next().unwrap()
    }

    pub fn pop(mut self) -> (T, Vec<T>) {
        // Can be unwrap_unchecked() but I'm scared tbh.
        (self.0.pop().unwrap(), self.0)
    }

    pub fn pop_nonlast(&mut self) -> T {
        assert!(self.len().get() > 1, "pop_nonlast requires at least 2 elements");
        self.0.pop().unwrap()
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }

    pub fn split_last(&self) -> (&[T], &T) {
        (&self.0[..self.len().get() - 1], self.last())
    }

    pub fn len(&self) -> NonZeroUsize {
        // Can be new_unchecked
        NonZeroUsize::new(self.0.len()).unwrap()
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> NonEmptyVec<U> {
        // SAFETY: Length is known to be non-zero, since self is a NonEmptyVec.
        unsafe { NonEmptyVec::from_iter_unchecked(self.into_iter().map(f)) }
    }

    pub fn map_ref<U>(&self, f: impl FnMut(&T) -> U) -> NonEmptyVec<U> {
        // SAFETY: Length is known to be non-zero, since self is a NonEmptyVec.
        unsafe { NonEmptyVec::from_iter_unchecked(self.iter().map(f)) }
    }
}

impl<T> Deref for NonEmptyVec<T> {
    type Target = [T];
    fn deref(&self) -> &'_ Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NonEmptyVec<T> {
    fn deref_mut(&mut self) -> &'_ mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "quickcheck")]
impl<T: quickcheck::Arbitrary> quickcheck::Arbitrary for NonEmptyVec<T> {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let gen_size = std::cmp::max(g.size(), 1);
        let vec_size = usize::arbitrary(g) % gen_size + 1;
        let vec = (0..vec_size).map(|_| T::arbitrary(g)).collect();
        Self(vec)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.0
                .shrink()
                .map(NonEmptyVec::try_new)
                .filter_map(Result::ok),
        )
    }
}

impl<T> IntoIterator for NonEmptyVec<T> {
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a NonEmptyVec<T> {
    type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut NonEmptyVec<T> {
    type IntoIter = <&'a mut Vec<T> as IntoIterator>::IntoIter;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<T> From<NonEmptyVec<T>> for Vec<T> {
    fn from(v: NonEmptyVec<T>) -> Self {
        v.0
    }
}

#[macro_export]
macro_rules! ne_vec {
    ($($x:expr),+ $(,)?) => (
        // SAFETY: This is safe because the macro is only called with at least one argument.
        unsafe { $crate::NonEmptyVec::new_unchecked(vec![$($x),+]) }
    );
}

impl<T: Default> Default for NonEmptyVec<T> {
    fn default() -> Self {
        NonEmptyVec::of_single(T::default())
    }
}
