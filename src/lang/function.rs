use std::{fmt, rc::Rc};

use super::{EvalContext, LuaValue, EvalError};

#[derive(Clone)]
pub struct LuaFunction(Rc<dyn Fn(&mut dyn EvalContext, &[LuaValue]) -> Result<LuaValue, EvalError>>);

impl LuaFunction {
    pub fn new(func: impl Fn(&mut dyn EvalContext, &[LuaValue]) -> Result<LuaValue, EvalError> + 'static) -> Self {
        Self(Rc::new(func))
    }

    pub fn call(&self, context: &mut dyn EvalContext, args: &[LuaValue]) -> Result<LuaValue, EvalError> {
        self.0(context, args)
    }
}

impl fmt::Debug for LuaFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("<lua function>", f)
    }
}

impl PartialEq for LuaFunction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for LuaFunction {}
