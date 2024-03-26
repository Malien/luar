pub mod compiler;
pub mod eval;
pub mod syn;

pub use compiler::*;
pub use eval::eval_module;
pub use eval::call_function;

use crate::lang::LuaValue;

#[derive(Default)]
pub struct Context {
    pub globals: GlobalValues,
    stack: Vec<LuaValue>,
}

pub mod stdlib {
    use super::Context;

    pub fn std_context() -> Context {
        Context::default()
    }
}
