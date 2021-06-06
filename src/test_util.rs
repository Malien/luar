use quickcheck::Arbitrary;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct NonShrinkable<T>(pub T);

impl<T> Arbitrary for NonShrinkable<T>
where
    T: Arbitrary,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        NonShrinkable(T::arbitrary(g))
    }
}

macro_rules! deref_t {
    ($t:ident) => {
        impl<T> Deref for $t<T> {
            type Target = T;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> DerefMut for $t<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

deref_t!(NonShrinkable);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Finite<T>(pub T);

impl<T> Arbitrary for Finite<T>
where
    T: num::Float + Arbitrary,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        loop {
            let val = T::arbitrary(g);
            if val.is_finite() {
                return Finite(val);
            }
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.0
                .shrink()
                .filter(|v| v.is_finite())
                .map(|v| Finite(v)),
        )
    }
}

deref_t!(Finite);
