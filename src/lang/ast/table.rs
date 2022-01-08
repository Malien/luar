use crate::{
    lang::{Eval, EvalContext, EvalError, LuaValue},
    syn::TableConstructor,
};

impl Eval for TableConstructor {
    type Return = LuaValue;

    fn eval<Context>(&self, _: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        todo!();
    }
}
