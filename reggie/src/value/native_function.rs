use std::{hash::Hash, ops::Deref, rc::Rc};

use crate::{FFIFunc, FromArgs, NativeFunctionCallable, NativeFunctionWrapper};

#[derive(Clone, Debug)]
pub struct NativeFunction(pub(crate) Rc<NativeFunctionKind>);

impl Hash for NativeFunction {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for NativeFunction {}

impl<T> From<T> for NativeFunction
where
    T: NativeFunctionCallable + 'static,
{
    fn from(func: T) -> Self {
        Self(Rc::new(NativeFunctionKind(Box::new(func))))
    }
}

impl NativeFunction {
    pub fn new<'a, F, Args>(func: F) -> Self
    where
        F: FFIFunc<Args> + 'static,
        Args: FromArgs<'a> + 'static,
    {
        Self(Rc::new(NativeFunctionKind(Box::new(
            NativeFunctionWrapper::new(func),
        ))))
    }
}

pub(crate) struct NativeFunctionKind(Box<dyn NativeFunctionCallable>);

impl Deref for NativeFunctionKind {
    type Target = dyn NativeFunctionCallable;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl std::fmt::Debug for NativeFunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = self.0.as_ref() as *const _;
        write!(f, "Dyn {:p}", ptr)
    }
}
