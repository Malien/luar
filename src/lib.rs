#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod util;

pub mod error;
pub mod lang;
pub mod stdlib;
pub mod syn;
pub mod ast_vm;
pub mod reggie;