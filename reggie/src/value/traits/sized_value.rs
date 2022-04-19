use crate::LuaValue;

pub trait SizedValue {
    const COUNT: usize;
}

macro_rules! count {
    () => { 0usize };
    ($x:tt $($xs:tt)*) => (1usize + count!($($xs) *))
}

macro_rules! impl_return_size_tuple {
    ($($gen: ident) +) => {
        impl<$($gen),+> SizedValue for ($($gen,)+) {
            const COUNT: usize = count!($($gen) +);
        }
    };
}

impl SizedValue for () {
    const COUNT: usize = 0;
}

impl SizedValue for LuaValue {
    const COUNT: usize = 1;
}

impl<'a, T: SizedValue> SizedValue for &'a T {
    const COUNT: usize = T::COUNT;
}

impl SizedValue for bool {
    const COUNT: usize = 1;
}

impl_return_size_tuple! { A }
impl_return_size_tuple! { A B }
impl_return_size_tuple! { A B C }
impl_return_size_tuple! { A B C D }
