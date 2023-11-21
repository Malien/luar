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

impl<Func, A0, A1, Ret> FFIFunc<(A0, A1)> for Func
where 
    Func: Fn(A0, A1) -> Ret,
    Ret: ReturnRepresentable,
{
    // type Args = (A0, A1);
    type Output = Ret;
    fn call(&self, (arg0, arg1): (A0, A1)) -> Ret {
        (self)(arg0, arg1)
    }
}
