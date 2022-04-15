#[macro_export]
macro_rules! conditional_tests {
    ($engine: ty, $context: expr) => {
        mod conditional {
            use ::luar::error::LuaError;
            use ::luar::lang::{Engine, ReturnValue, LuaValue};
            use ::luar::syn::lua_parser;
            use ::luar::ne_vec;

            #[test]
            fn if_with_falsy_condition_does_not_evaluate_body() -> Result<(), LuaError> {
                let module = lua_parser::module(
                    "if nil then
                        result = 1
                    end
                    return result",
                )?;
                let mut context = $context;
                let res = <$engine>::eval_module(&module, &mut context)?;
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
                let mut context = $context;
                let res = <$engine>::eval_module(&module, &mut context)?;
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
                let mut context = $context;
                let res = <$engine>::eval_module(&module, &mut context)?;
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
                let mut context = $context;
                let res = <$engine>::eval_module(&module, &mut context)?;
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
                let mut context = $context;
                let res = <$engine>::eval_module(&module, &mut context)?;
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
                let mut context = $context;
                let res = <$engine>::eval_module(&module, &mut context)?;
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
                let mut context = $context;
                let res = <$engine>::eval_module(&module, &mut context)?;
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
                let mut context = $context;
                let res = <$engine>::eval_module(&module, &mut context)?;
                assert_eq!(res, ReturnValue::string("else branch"));
                Ok(())
            }
        }
    };
}
