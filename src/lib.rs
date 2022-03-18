#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
mod test_util;

pub mod error;
pub mod lang;
pub mod lex;
pub mod stdlib;
pub mod syn;
mod util;