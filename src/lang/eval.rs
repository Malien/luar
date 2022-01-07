use super::{EvalContext, EvalError};

pub trait Eval {
    type Return;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError>;
}

impl<T: Eval> Eval for Box<T> {
    type Return = T::Return;
    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        T::eval(self, context)
    }
}
