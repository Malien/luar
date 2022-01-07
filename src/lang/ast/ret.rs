use crate::{lang::{Eval, LuaValue, EvalContext, EvalError}, syn::Return};

impl Eval for Return {
    type Return = LuaValue;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self.0 {
            Some(ref expr) => expr.eval(context),
            None => Ok(LuaValue::Nil),
        }
    }
}
