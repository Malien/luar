use crate::{
    lang::{Eval, EvalContext, EvalContextExt, EvalError, LuaValue},
    syn::Var,
};

impl Eval for Var {
    type Return = LuaValue;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        match self {
            Self::Named(ident) => Ok(context.get(ident).clone()),
            _ => todo!(),
        }
    }
}
