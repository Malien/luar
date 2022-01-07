use crate::{
    lang::{Eval, EvalContext, EvalContextExt, EvalError, LuaValue},
    syn::Var,
};

impl Eval for Var {
    type Return = LuaValue;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self {
            Self::Named(ident) => Ok(context.get(ident).clone()),
            _ => todo!(),
        }
    }
}
