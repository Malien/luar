use std::{fmt, hash::Hash, rc::Rc};

use crate::{lang::{Context, LuaValue, ReturnValue}, EvalError, opt};

type NativeFn = dyn Fn(&mut Context, &[LuaValue]) -> Result<ReturnValue, EvalError>;

#[derive(Clone)]
pub struct NativeFunction(Rc<NativeFn>);

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native function: {:p}", Rc::as_ptr(&self.0))
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for NativeFunction {}

impl Hash for NativeFunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl NativeFunction {
    pub fn new(
        func: impl Fn(&mut Context, &[LuaValue]) -> Result<ReturnValue, EvalError> + 'static,
    ) -> Self {
        Self(Rc::new(func))
    }

    pub fn call(
        &self,
        context: &mut Context,
        args: &[LuaValue],
    ) -> Result<ReturnValue, EvalError> {
        self.0(context, args)
    }

    pub fn addr(&self) -> *const NativeFn {
        Rc::as_ptr(&self.0)
    }
}

pub struct InnerFn {
    pub local_count: u16,
    pub arg_count: u16,
    pub body: opt::syn::Block,
}

#[derive(Clone)]
pub struct LuaFunction(pub(crate) Rc<InnerFn>);

impl LuaFunction {
    pub fn new(
        decl: &opt::syn::FunctionDeclaration,
    ) -> Self {
        Self(Rc::new(InnerFn {
            local_count: decl.local_count,
            arg_count: decl.arg_count,
            body: decl.body.clone(),
        }))
    }

    pub fn addr(&self) -> *const InnerFn {
        Rc::as_ptr(&self.0)
    }
}

impl fmt::Debug for LuaFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lua function: {:p}", Rc::as_ptr(&self.0))
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
