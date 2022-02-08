use crate::{
    lang::{ControlFlow, Eval},
    syn::{Conditional, ConditionalTail},
};

impl Eval for Conditional {
    type Return = ControlFlow;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, crate::lang::EvalError>
    where
        Context: crate::lang::EvalContext + ?Sized,
    {
        let Conditional {
            condition,
            body,
            tail,
        } = self;

        if condition.eval(context)?.is_truthy() {
            body.eval(context)
        } else {
            match tail {
                ConditionalTail::End => Ok(ControlFlow::Continue),
                ConditionalTail::Else(block) => block.eval(context),
                ConditionalTail::ElseIf(condition) => condition.eval(context),
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{
        error::LuaError,
        lang::{Eval, GlobalContext, LuaValue},
        ne_vec,
        syn::string_parser,
    };

    #[test]
    fn if_with_falsy_condition_does_not_evaluate_body() -> Result<(), LuaError> {
        let module = string_parser::module(
            "if nil then
                result = 1
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert!(res.is_falsy());
        Ok(())
    }

    #[test]
    fn if_with_truthy_condition_evaluates_body() -> Result<(), LuaError> {
        let module = string_parser::module(
            "if 1 then
                result = 1
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert!(res.is_truthy());
        Ok(())
    }

    #[test]
    fn if_with_truthy_condition_does_not_evaluate_else_branch() -> Result<(), LuaError> {
        let module = string_parser::module(
            "if 1 then
                result = 'true branch'
            else
                result = 'false branch'
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert_eq!(res, LuaValue::String("true branch".to_string()));
        Ok(())
    }

    #[test]
    fn if_with_falsy_condition_evaluates_else_branch() -> Result<(), LuaError> {
        let module = string_parser::module(
            "if nil then
                result = 'true branch'
            else
                result = 'false branch'
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert_eq!(res, LuaValue::String("false branch".to_string()));
        Ok(())
    }

    #[test]
    fn if_with_truthy_condition_does_not_evaluate_elseif_branch() -> Result<(), LuaError> {
        let module = string_parser::module(
            "function side_effect() 
                side_effect_committed = 1
            end
            
            if 1 then
                result = 'if branch'
            elseif side_effect() then
                result = 'elseif branch'
            else
                result = 'else branch'
            end
            return result, side_effect_committed",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        let expected =
            LuaValue::MultiValue(ne_vec![LuaValue::String("if branch".into()), LuaValue::Nil]);
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn if_with_falsy_condition_and_passing_elseif_should_evaluate_elseif_branch(
    ) -> Result<(), LuaError> {
        let module = string_parser::module(
            "if nil then
                result = 'if branch'
            elseif 1 then
                result = 'elseif branch'
            else
                result = 'else branch'
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert_eq!(res, LuaValue::String("elseif branch".to_string()));
        Ok(())
    }

    #[test]
    fn if_with_falsy_condition_and_falsy_elseif_condition_should_not_evaluate_anything(
    ) -> Result<(), LuaError> {
        let module = string_parser::module(
            "if nil then
                result = 'if branch'
            elseif nil then
                result = 'elseif branch'
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert_eq!(res, LuaValue::Nil);
        Ok(())
    }

    #[test]
    fn if_with_falsy_condition_and_falsy_elseif_condition_should_evaluate_else_branch(
    ) -> Result<(), LuaError> {
        let module = string_parser::module(
            "if nil then
                result = 'if branch'
            elseif nil then
                result = 'elseif branch'
            else 
                result = 'else branch'
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert_eq!(res, LuaValue::String("else branch".into()));
        Ok(())
    }
}
