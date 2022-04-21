use crate::LuaValue;

pub trait SizedValue {
    const COUNT: u16;
}

macro_rules! count {
    () => { 0 };
    ($x:tt $($xs:tt)*) => (1 + count!($($xs) *))
}

macro_rules! impl_return_size_tuple {
    ($($gen: ident) +) => {
        impl<$($gen),+> SizedValue for ($($gen,)+) {
            const COUNT: u16 = count!($($gen) +);
        }
    };
}

impl SizedValue for () {
    const COUNT: u16 = 0;
}

impl SizedValue for LuaValue {
    const COUNT: u16 = 1;
}

impl<'a, T: SizedValue> SizedValue for &'a T {
    const COUNT: u16 = T::COUNT;
}

impl SizedValue for bool {
    const COUNT: u16 = 1;
}

impl_return_size_tuple! { A }
impl_return_size_tuple! { A B }
impl_return_size_tuple! { A B C }
impl_return_size_tuple! { A B C D }
