use crate::{
    lang::{
        ast::{eval_block, eval_expr},
        ControlFlow, EvalError, LocalScope, ScopeHolder,
    },
    syn::{Conditional, ConditionalTail},
};

pub(crate) fn eval_conditional(
    conditional: &Conditional,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ControlFlow, EvalError> {
    let Conditional {
        condition,
        body,
        tail,
    } = conditional;

    if eval_expr(condition, scope)?.first_value().is_truthy() {
        eval_block(body, &mut scope.child_scope())
    } else {
        match tail {
            ConditionalTail::End => Ok(ControlFlow::Continue),
            ConditionalTail::Else(block) => eval_block(block, &mut scope.child_scope()),
            ConditionalTail::ElseIf(condition) => eval_conditional(condition, scope),
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{
        error::LuaError,
        lang::{ast, GlobalContext, LuaValue, ReturnValue},
        ne_vec,
        syn::lua_parser,
    };

    #[test]
    fn if_with_falsy_condition_does_not_evaluate_body() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "if nil then
                result = 1
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = ast::eval_module(&module, &mut context)?;
        assert!(res.assert_single().is_falsy());
        Ok(())
    }

    #[test]
    fn if_with_truthy_condition_evaluates_body() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "if 1 then
                result = 1
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = ast::eval_module(&module, &mut context)?;
        assert!(res.assert_single().is_truthy());
        Ok(())
    }

    #[test]
    fn if_with_truthy_condition_does_not_evaluate_else_branch() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "if 1 then
                result = 'true branch'
            else
                result = 'false branch'
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = ast::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::string("true branch"));
        Ok(())
    }

    #[test]
    fn if_with_falsy_condition_evaluates_else_branch() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "if nil then
                result = 'true branch'
            else
                result = 'false branch'
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = ast::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::string("false branch"));
        Ok(())
    }

    #[test]
    fn if_with_truthy_condition_does_not_evaluate_elseif_branch() -> Result<(), LuaError> {
        let module = lua_parser::module(
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
        let res = ast::eval_module(&module, &mut context)?;
        let expected =
            ReturnValue::MultiValue(ne_vec![LuaValue::string("if branch"), LuaValue::Nil]);
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn if_with_falsy_condition_and_passing_elseif_should_evaluate_elseif_branch(
    ) -> Result<(), LuaError> {
        let module = lua_parser::module(
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
        let res = ast::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::string("elseif branch"));
        Ok(())
    }

    #[test]
    fn if_with_falsy_condition_and_falsy_elseif_condition_should_not_evaluate_anything(
    ) -> Result<(), LuaError> {
        let module = lua_parser::module(
            "if nil then
                result = 'if branch'
            elseif nil then
                result = 'elseif branch'
            end
            return result",
        )?;
        let mut context = GlobalContext::new();
        let res = ast::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::Nil);
        Ok(())
    }

    #[test]
    fn if_with_falsy_condition_and_falsy_elseif_condition_should_evaluate_else_branch(
    ) -> Result<(), LuaError> {
        let module = lua_parser::module(
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
        let res = ast::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::string("else branch"));
        Ok(())
    }
}
