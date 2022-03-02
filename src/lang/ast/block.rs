use crate::{
    lang::{ControlFlow, Eval, EvalContext, EvalError, LocalContext},
    syn::Block,
};

impl Eval for Block {
    type Return = ControlFlow;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        let local_ctx: &mut dyn EvalContext = &mut LocalContext::new(context);
        for statement in &self.statements {
            if let ControlFlow::Return(value) = statement.eval(local_ctx)? {
                return Ok(ControlFlow::Return(value))
            }
        }
        self.ret
            .as_ref()
            .map(|ret| ret.eval(local_ctx).map(ControlFlow::Return))
            .unwrap_or(Ok(ControlFlow::Continue))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::LuaError,
        lang::{Eval, GlobalContext},
        syn::string_parser,
    };

    #[test]
    fn early_returns_from_blocks_stop_flow_of_execution() -> Result<(), LuaError> {
        let module = string_parser::module(
            "if 1 then
                return 1
            end
            return nil",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert!(res.assert_single().is_truthy());
        Ok(())
    }

    #[test]
    fn early_returns_from_functions_stop_flow_of_execution() -> Result<(), LuaError> {
        let module = string_parser::module(
            "function fn()
                if 1 then
                    return 1
                end
                return nil
            end
            return fn()",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert!(res.assert_single().is_truthy());
        Ok(())
    }
}
