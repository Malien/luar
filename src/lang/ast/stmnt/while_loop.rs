use crate::{
    lang::{ControlFlow, Eval},
    syn::WhileLoop,
};

impl Eval for WhileLoop {
    type Return = ControlFlow;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, crate::lang::EvalError>
    where
        Context: crate::lang::EvalContext + ?Sized,
    {
        let Self { condition, body } = self;
        while condition.eval(context)?.is_truthy() {
            if let ControlFlow::Return(ret_value) = body.eval(context)? {
                return Ok(ControlFlow::Return(ret_value));
            }
        }
        Ok(ControlFlow::Continue)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::LuaError,
        lang::{Eval, GlobalContext, LuaValue, EvalContextExt},
        syn::string_parser, ne_vec,
    };

    #[test]
    fn while_loop_with_falsy_condition_does_not_execute_body() -> Result<(), LuaError> {
        let module = string_parser::module(
            "while nil do
                side_effect_committed = 1
            end
            return side_effect_committed",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert!(res.is_falsy());
        Ok(())
    }

    #[test]
    fn while_loop_with_truthy_condition_executes_body_at_least_once() -> Result<(), LuaError> {
        let module = string_parser::module(
            "while not side_effect_committed do
                side_effect_committed = 1
            end
            return side_effect_committed",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert!(res.is_truthy());
        Ok(())
    }

    #[quickcheck]
    fn while_loop_executes_until_condition_is_true(times: u8) -> Result<(), LuaError> {
        let module = string_parser::module(
            "count_executed = 0
            while i ~= 0 do
                count_executed = count_executed + 1
                i = i - 1
            end
            return i, count_executed"
        )?;
        let mut context = GlobalContext::new();
        context.set("i", LuaValue::number(times));
        let res = module.eval(&mut context)?;
        let expected = LuaValue::MultiValue(ne_vec![
            LuaValue::number(0),
            LuaValue::number(times)
        ]);
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn while_loop_early_return() -> Result<(), LuaError> {
        let module = string_parser::module(
            "while 1 do
                return 1
            end"
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert!(res.is_truthy());
        Ok(())
    }
}
