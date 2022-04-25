use std::{rc::Rc, hash::Hash};

use crate::{
    signature::{ArgumentType, FunctionSignatureList},
    FFIFunc, FromArgs, LuaValue, NativeFunctionCallable, NativeFunctionWrapper,
    ReturnRepresentable,
};

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

impl From<OverloadSet> for NativeFunction {
    fn from(set: OverloadSet) -> Self {
        Self(Rc::new(NativeFunctionKind::OverloadSet(set)))
    }
}

impl<T> From<T> for NativeFunction
where
    T: NativeFunctionCallable + 'static,
{
    fn from(func: T) -> Self {
        Self(Rc::new(NativeFunctionKind::Dyn(Box::new(func))))
    }
}

impl NativeFunction {
    pub fn new<F, Args>(func: F) -> Self
    where
        F: FFIFunc<Args> + 'static,
        Args: FromArgs + 'static,
    {
        Self(Rc::new(NativeFunctionKind::Dyn(Box::new(
            NativeFunctionWrapper::new(func),
        ))))
    }
}

pub(crate) enum NativeFunctionKind {
    Dyn(Box<dyn NativeFunctionCallable>),
    OverloadSet(OverloadSet),
}

impl std::fmt::Debug for NativeFunctionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dyn(_) => f.write_str("Dyn"),
            Self::OverloadSet(set) => f.debug_tuple("OverloadSet").field(set).finish(),
        }
    }
}

#[derive(Debug)]
pub struct OverloadSet {
    pub(crate) rules: Vec<OverloadRule>,
}

impl OverloadSet {
    // pub fn lookup_rule(&self, args: &[ArgumentType]) -> Option<OverloadRule> {
    //     for rule in self.rules {
    //         rule.arguments
    //     }
    // }
}

#[derive(Debug)]
pub struct OverloadRule {
    pub(crate) arguments: FunctionSignatureList,
    pub(crate) returns: FunctionSignatureList,
    pub(crate) error_prone: bool,
    pub(crate) func: *const (),
    // intrinsics: Vec<Instruction>
}

trait FromLua {
    fn from_lua(value: LuaValue) -> Self;
}

trait IntoLua {
    fn into_lua(self) -> LuaValue;
    fn lua_type() -> ArgumentType;
}

impl<T> From<fn() -> T> for OverloadRule
where
    T: ReturnRepresentable,
{
    fn from(func: fn() -> T) -> Self {
        Self {
            arguments: FunctionSignatureList::Finite(vec![]),
            func: func as *const (),
            error_prone: false,
            returns: T::returns(),
        }
    }
}

impl<T, U> From<fn(T) -> U> for OverloadRule
where
    T: FromLua + IntoLua,
    U: ReturnRepresentable,
{
    fn from(func: fn(T) -> U) -> Self {
        Self {
            arguments: FunctionSignatureList::Finite(vec![T::lua_type()]),
            returns: U::returns(),
            error_prone: false,
            func: func as *const (),
        }
    }
}

impl OverloadSet {
    pub const fn new(rules: Vec<OverloadRule>) -> Self {
        Self { rules }
    }
}

impl FromLua for LuaValue {
    fn from_lua(value: LuaValue) -> Self {
        value
    }
}

impl IntoLua for LuaValue {
    fn into_lua(self) -> LuaValue {
        self
    }

    fn lua_type() -> ArgumentType {
        ArgumentType::Dynamic
    }
}
