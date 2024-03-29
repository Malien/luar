use std::{fmt, hash::Hash, rc::Rc};

use crate::{lang::{GlobalContext, LuaValue, ReturnValue}, EvalError};

pub type InnerFn = dyn Fn(&mut GlobalContext, &[LuaValue]) -> Result<ReturnValue, EvalError>;

#[derive(Clone)]
pub struct LuaFunction(Rc<InnerFn>);

impl LuaFunction {
    pub fn new(
        func: impl Fn(&mut GlobalContext, &[LuaValue]) -> Result<ReturnValue, EvalError> + 'static,
    ) -> Self {
        Self(Rc::new(func))
    }

    pub fn call(
        &self,
        context: &mut GlobalContext,
        args: &[LuaValue],
    ) -> Result<ReturnValue, EvalError> {
        self.0(context, args)
    }

    pub fn addr(&self) -> *const InnerFn {
        Rc::as_ptr(&self.0)
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

impl Hash for LuaFunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl Eq for LuaFunction {}
