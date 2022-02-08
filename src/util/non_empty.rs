use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyVec<T>(Vec<T>);

pub struct VecIsEmptyError<T>(Vec<T>);

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

    pub fn first(&self) -> &'_ T {
        // Can be unwrap_unchecked() but I'm scared tbh.
        self.0.first().unwrap()
    }

    pub fn last(&self) -> &'_ T {
        // Can be unwrap_unchecked() but I'm scared tbh.
        self.0.last().unwrap()
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
}

impl<T> Deref for NonEmptyVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &'_ Self::Target {
        &self.0
    }
}

#[cfg(test)]
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
        unsafe { crate::util::NonEmptyVec::new_unchecked(vec![$($x),+]) }
    );
}
