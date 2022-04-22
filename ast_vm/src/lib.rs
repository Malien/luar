#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod util;

pub mod lang;
pub mod stdlib;
pub mod ast_vm;

pub use ast_vm::*;

use lang::LuaValue;
pub type LuaError = luar_error::LuaError<LuaValue>;
pub type EvalError = luar_error::EvalError<LuaValue>;
pub type TypeError = luar_error::TypeError<LuaValue>;
pub type ArithmeticError = luar_error::ArithmeticError<LuaValue>;