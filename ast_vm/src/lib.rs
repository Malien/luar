#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod util;

pub mod lang;
pub mod stdlib;

mod block;
pub(crate) use block::*;

mod expr;
pub(crate) use expr::*;

mod fn_decl;
pub(crate) use fn_decl::*;

mod module;
use luar_string::LuaString;
pub use module::*;

mod ret;
pub(crate) use ret::*;

mod stmnt;
pub(crate) use stmnt::*;

mod var;
pub(crate) use var::*;

mod tail_values;
pub use tail_values::*;

pub mod scope;

pub mod opt;

mod ctrl_flow;
pub use ctrl_flow::*;

use lang::LuaValue;
pub type LuaError = luar_error::LuaError<LuaValue, LuaString>;
pub type EvalError = luar_error::EvalError<LuaValue, LuaString>;
pub type TypeError = luar_error::TypeError<LuaValue>;
pub type ArithmeticError = luar_error::ArithmeticError<LuaValue>;

#[cfg(test)]
pub(crate) fn vec_of_idents(len: usize, prefix: &str) -> Vec<luar_lex::Ident> {
    (0..len)
        .into_iter()
        .map(|i| format!("{}{}", prefix, i))
        .map(luar_lex::Ident::new)
        .collect()
}
