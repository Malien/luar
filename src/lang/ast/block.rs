use crate::{
    lang::{Eval, EvalContext, EvalError, LuaValue},
    syn::Block,
};

impl Eval for Block {
    type Return = LuaValue;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        for statement in &self.statements {
            statement.eval(context)?;
        }
        self.ret
            .as_ref()
            .map(|ret| ret.eval(context))
            .unwrap_or(Ok(LuaValue::Nil))
    }
}
