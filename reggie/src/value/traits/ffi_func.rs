use crate::ReturnRepresentable;

pub trait FFIFunc<Args> {
    // type Args: FromArgs;
    type Output: ReturnRepresentable;
    fn call(&self, args: Args) -> Self::Output;
}

impl<Func, Ret> FFIFunc<()> for Func
where
    Func: Fn() -> Ret,
    Ret: ReturnRepresentable,
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
    Ret: ReturnRepresentable,
{
    // type Args = (Arg,);
    type Output = Ret;
    fn call(&self, (arg0,): (Arg,)) -> Ret {
        (self)(arg0)
    }
}
