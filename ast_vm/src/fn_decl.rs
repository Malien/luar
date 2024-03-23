use crate::lang::{
    FunctionContext, GlobalContext, LocalScope, LuaValue, NativeFunction, ScopeHolder,
};
use luar_lex::Ident;
use luar_syn::{FunctionDeclaration, FunctionName};

use super::{assign_to_var, eval_block, ControlFlow};

pub(crate) fn eval_fn_decl(
    decl: &FunctionDeclaration,
    scope: &mut LocalScope<GlobalContext>,
) -> Result<(), crate::EvalError> {
    match &decl.name {
        FunctionName::Plain(var) => {
            let function = NativeFunction::new({
                let body = decl.body.clone();
                let arg_names = decl.args.clone();
                move |context, args| {
                    let mut fn_ctx = FunctionContext::new(context);
                    let mut scope = fn_ctx.top_level_scope();
                    declare_arguments(&mut scope, &arg_names, args);
                    eval_block(&body, &mut scope).map(ControlFlow::function_return)
                }
            });
            assign_to_var(scope, var, LuaValue::NativeFunction(function))
        }
        FunctionName::Method(base, name) => {
            todo!("Cannot evaluate method declaration for {base}:{name} yet")
        }
    }
}

fn declare_arguments(scope: &mut LocalScope<impl ScopeHolder>, names: &[Ident], args: &[LuaValue]) {
    let iter = names.into_iter().cloned().zip(
        args.iter()
            .cloned()
            .chain(std::iter::repeat_with(|| LuaValue::Nil)),
    );

    for (name, value) in iter {
        scope.declare_local(name, value);
    }
}

#[cfg(test)]
mod test {
    use crate as ast_vm;
    use crate::{
        lang::{GlobalContext, LuaValue, ReturnValue},
        LuaError,
    };
    use itertools::Itertools;
    use luar_lex::Ident;
    use luar_syn::{
        lua_parser, Block, Chunk, Expression, FunctionCall, FunctionCallArgs, FunctionDeclaration,
        FunctionName, Module, Return, Var,
    };
    use non_empty::NonEmptyVec;
    use smallvec::smallvec;

    #[quickcheck]
    fn fn_declaration_puts_function_in_scope(ident: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("function {}() end", ident))?;
        let mut context = GlobalContext::new();
        ast_vm::eval_module(&module, &mut context)?;
        assert!(context.get(&ident).is_function());
        Ok(())
    }

    #[quickcheck]
    fn fn_declaration_return(ret_value: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function myfn() return value end
            return myfn()",
        )?;
        let mut context = GlobalContext::new();
        context.set("value", ret_value.clone());
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(context.get("myfn").is_function());
        assert!(res.assert_single().total_eq(&ret_value));

        Ok(())
    }

    #[quickcheck]
    #[ignore = "This would generate unsupported operations for now, so as to always fail"]
    fn running_block_in_function_is_the_same_as_running_it_in_global_context(
        block: Block,
    ) -> Result<(), LuaError> {
        let name = Var::Named(Ident::new("myfn"));
        let fn_module = Module {
            chunks: vec![Chunk::FnDecl(FunctionDeclaration {
                name: FunctionName::Plain(name.clone()),
                args: vec![],
                body: block.clone(),
            })],
            ret: Some(Return::single(Expression::FunctionCall(
                FunctionCall::Function {
                    args: FunctionCallArgs::Arglist(vec![]),
                    func: name,
                },
            ))),
        };
        let block_module = Module {
            chunks: block.statements.into_iter().map(Chunk::Statement).collect(),
            ret: block.ret,
        };

        let mut context = GlobalContext::new();
        let res_block = ast_vm::eval_module(&block_module, &mut context);
        context = GlobalContext::new();
        let res_fn = ast_vm::eval_module(&fn_module, &mut context);

        let is_same = match (res_block, res_fn) {
            (Err(_), Err(_)) => true,
            (Ok(l), Ok(r)) => l.total_eq(&r),
            _ => false,
        };

        assert!(is_same);

        Ok(())
    }

    #[quickcheck]
    fn function_multiple_returns(values: NonEmptyVec<LuaValue>) -> Result<(), LuaError> {
        let idents: Vec<_> = (0..values.len().get())
            .into_iter()
            .map(|i| format!("value{}", i))
            .map(Ident::new)
            .collect();
        let idents_str = idents.iter().join(", ");
        let module = lua_parser::module(&format!(
            "function myfn()
                return {}
            end
            return myfn()",
            idents_str
        ))?;
        let mut context = GlobalContext::new();
        for (value, ident) in values.iter().zip(idents) {
            context.set(ident, value.clone());
        }
        let res = ast_vm::eval_module(&module, &mut context)?;
        let expected: ReturnValue = values.into_iter().collect();
        assert!(res.total_eq(&expected));
        Ok(())
    }

    #[test]
    fn function_executes_side_effect() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "executed = nil
            function myfn() 
                executed = 1
            end
            myfn()
            return executed",
        )?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.assert_single().is_truthy());
        Ok(())
    }

    #[quickcheck]
    fn local_declarations_stay_local(ident: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!(
            "{} = \"global\"
            function myfn()
                local {} = \"local\"
                return {}
            end
            return myfn(), {}",
            ident, ident, ident, ident
        ))?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        let expected = ReturnValue(smallvec![
            LuaValue::string("local"),
            LuaValue::string("global"),
        ]);
        assert_eq!(res, expected);

        Ok(())
    }

    #[quickcheck]
    fn arguments_passed_in_are_defined_as_local_variables_inside_fn(
        value: LuaValue,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function myfn(arg)
                return arg
            end
            return myfn(value), arg",
        )?;
        let mut context = GlobalContext::new();
        context.set("value", value.clone());
        let res = ast_vm::eval_module(&module, &mut context)?;
        let expected = ReturnValue(smallvec![value, LuaValue::Nil]);
        assert!(res.total_eq(&expected));
        Ok(())
    }

    #[test]
    fn not_passed_arguments_are_set_to_nil() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function myfn(a, b, c, d)
                return a, b, c, d
            end
            return myfn(1, 2)",
        )?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        let expected = ReturnValue(smallvec![
            LuaValue::number(1i32),
            LuaValue::number(2i32),
            LuaValue::Nil,
            LuaValue::Nil
        ]);
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn passing_more_arguments_than_stated_just_gets_arglist_truncated() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function myfn(a, b)
                return a, b
            end
            return myfn(1, 2, 3, 4)",
        )?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        let expected = ReturnValue(smallvec![LuaValue::number(1i32), LuaValue::number(2i32)]);
        assert_eq!(res, expected);
        Ok(())
    }

    macro_rules! mv_num {
        ($($num:expr),*) => {
            ReturnValue(smallvec![$(LuaValue::number($num as i32),)*])
        };
    }

    #[test]
    fn multiple_return_is_propagated() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function mult() 
                return 1, 2
            end
            function m1()
                return mult()
            end
            function m2()
                return 3, mult()
            end
            function m3()
                return mult(), 3
            end",
        )?;
        let mut context = GlobalContext::new();
        ast_vm::eval_module(&module, &mut context)?;
        let expectations = [
            ("mult", mv_num![1, 2]),
            ("m1", mv_num![1, 2]),
            ("m2", mv_num![3, 1, 2]),
            ("m3", mv_num![1, 3]),
        ];
        for (func, expected) in expectations {
            let res = context
                .get(func)
                .unwrap_native_function_ref()
                .clone()
                .call(&mut context, &[])?;
            assert_eq!(res, expected);
        }

        Ok(())
    }
}
