use std::{marker::PhantomData, sync::Arc};

use crate::{EvalError, LuaValue, Machine};

#[derive(Clone, Debug)]
pub struct NativeFunction(pub(crate) Arc<NativeFunctionKind>);

impl From<OverloadSet> for NativeFunction {
    fn from(set: OverloadSet) -> Self {
        Self(Arc::new(NativeFunctionKind::OverloadSet(set)))
    }
}

impl<T> From<T> for NativeFunction
where
    T: NativeFunctionCallable + 'static,
{
    fn from(func: T) -> Self {
        Self(Arc::new(NativeFunctionKind::Dyn(Box::new(func))))
    }
}

impl NativeFunction {
    pub fn dyn_fn<F, Args>(func: F) -> Self
    where
        F: FFIFunc<Args> + 'static,
        Args: FromArgs + 'static,
    {
        Self(Arc::new(NativeFunctionKind::Dyn(Box::new(
            NativeFunctionWrapper {
                func,
                _args: PhantomData,
            },
        ))))
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for NativeFunction {}

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

#[derive(Debug)]
pub enum FunctionSignatureList {
    Finite(Vec<ArgumentType>),
    Unspecified,
}

#[derive(Debug)]
pub struct OverloadRule {
    pub(crate) arguments: FunctionSignatureList,
    pub(crate) returns: FunctionSignatureList,
    pub(crate) error_prone: bool,
    pub(crate) func: *const (),
    // intrinsics: Vec<Instruction>
}

#[derive(Debug)]
pub enum ArgumentType {
    Dynamic,
    Int,
    Float,
    String,
}

pub trait NativeFunctionCallable {
    fn call(&self, machine: &mut Machine);
    fn return_count(&self) -> usize;
}

trait Fn0 {
    type Output;

    fn eval(&self) -> Self::Output;
}

pub trait FromArgs {
    fn from_args(machine: &mut Machine, arg_count: u32) -> Self;
}

pub trait FFIFunc<Args> {
    // type Args: FromArgs;
    type Output: LuaReturnRepresentable;
    fn call(&self, args: Args) -> Self::Output;
}

impl<Func, Ret> FFIFunc<()> for Func
where
    Func: Fn() -> Ret,
    Ret: LuaReturnRepresentable,
{
    // type Args = ();
    type Output = Ret;
    fn call(&self, (): ()) -> Ret {
        (self)()
    }
}

impl<Func, Arg, Ret> FFIFunc<(Arg,)> for Func
where
    Func: Fn(Arg) -> Ret,
    Ret: LuaReturnRepresentable,
{
    // type Args = (Arg,);
    type Output = Ret;
    fn call(&self, (arg0,): (Arg,)) -> Ret {
        (self)(arg0)
    }
}

pub struct NativeFunctionWrapper<F, Args> {
    func: F,
    _args: PhantomData<Args>,
}

impl<F, Args> NativeFunctionCallable for NativeFunctionWrapper<F, Args>
where
    F: FFIFunc<Args>,
    Args: FromArgs,
{
    fn call(&self, machine: &mut Machine) {
        let args = Args::from_args(machine, machine.value_count);
        let res = self.func.call(args);
        // TODO: cause error from within FFI calls to propagate inside of the VM
        res.to_lua_return(machine).unwrap();
    }

    fn return_count(&self) -> usize {
        F::Output::return_count()
    }
}

pub trait LuaReturnRepresentable {
    fn returns() -> FunctionSignatureList;
    fn to_lua_return(self, machine: &mut Machine) -> Result<(), EvalError>;
    fn return_count() -> usize;
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
    T: LuaReturnRepresentable,
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
    U: LuaReturnRepresentable,
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

impl LuaReturnRepresentable for () {
    fn returns() -> FunctionSignatureList {
        FunctionSignatureList::Finite(vec![])
    }
    fn to_lua_return(self, _: &mut Machine) -> Result<(), EvalError> {
        Ok(())
    }

    fn return_count() -> usize {
        0
    }
}

impl LuaReturnRepresentable for LuaValue {
    fn returns() -> FunctionSignatureList {
        FunctionSignatureList::Finite(vec![ArgumentType::Dynamic])
    }
    fn to_lua_return(self, machine: &mut Machine) -> Result<(), EvalError> {
        machine.argument_registers.d[0] = self;
        Ok(())
    }
    fn return_count() -> usize {
        1
    }
}

impl<T> LuaReturnRepresentable for Result<T, EvalError>
where
    T: LuaReturnRepresentable,
{
    fn returns() -> FunctionSignatureList {
        T::returns()
    }
    fn to_lua_return(self, machine: &mut Machine) -> Result<(), EvalError> {
        self.and_then(|value| value.to_lua_return(machine))
    }
    fn return_count() -> usize {
        T::return_count()
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

impl FromArgs for () {
    fn from_args(_: &mut Machine, _: u32) -> Self {
        ()
    }
}

impl FromArgs for (LuaValue,) {
    fn from_args(machine: &mut Machine, argument_count: u32) -> Self {
        if argument_count > 0 {
            (machine.argument_registers.d[0].clone(),)
        } else {
            (LuaValue::Nil,)
        }
    }
}
