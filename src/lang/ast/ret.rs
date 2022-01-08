use crate::{
    lang::{Eval, EvalContext, EvalError, LuaValue},
    syn::Return,
};

impl Eval for Return {
    type Return = LuaValue;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        match self.0 {
            Some(ref expr) => expr.eval(context),
            None => Ok(LuaValue::Nil),
        }
    }
}
