use super::{EvalContext, EvalError};

pub trait Eval {
    type Return;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized;
}

impl<T: Eval> Eval for Box<T> {
    type Return = T::Return;
    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        T::eval(self, context)
    }
}
