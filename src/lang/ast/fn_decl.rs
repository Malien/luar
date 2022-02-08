use crate::{
    lang::{ControlFlow, Eval, EvalContext, LuaFunction, LuaValue},
    syn::{FunctionDeclaration, FunctionName},
};

use super::assign_to_var;

impl Eval for FunctionDeclaration {
    type Return = ();

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, crate::lang::EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        match &self.name {
            FunctionName::Plain(var) => {
                let function = LuaFunction::new({
                    let body = self.body.clone();
                    move |context, _| body.eval(context).map(ControlFlow::function_return)
                });
                assign_to_var(context, var, LuaValue::Function(function));
            }
            _ => todo!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::{
        error::LuaError,
        lang::{Eval, EvalContextExt, GlobalContext, LuaValue},
        lex::Ident,
        ne_vec,
        syn::{
            string_parser, Block, Chunk, Expression, FunctionCall, FunctionCallArgs,
            FunctionDeclaration, FunctionName, Module, Return, Var,
        },
        util::NonEmptyVec,
    };

    #[quickcheck]
    fn fn_declaration_puts_function_in_scope(ident: Ident) -> Result<(), LuaError> {
        let module = string_parser::module(&format!("function {}() end", ident))?;
        let mut context = GlobalContext::new();
        module.eval(&mut context)?;
        assert!(context.get(&ident).is_function());
        Ok(())
    }

    #[quickcheck]
    fn fn_declaration_return(ret_value: LuaValue) -> Result<(), LuaError> {
        let module = string_parser::module(
            "function myfn() return value end
            return myfn()",
        )?;
        let mut context = GlobalContext::new();
        context.set("value", ret_value.clone());
        let res = module.eval(&mut context)?;
        assert!(context.get("myfn").is_function());
        assert!(res.total_eq(&ret_value));

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
        let res_block = block_module.eval(&mut context);
        context = GlobalContext::new();
        let res_fn = fn_module.eval(&mut context);

        let is_same = match (res_block, res_fn) {
            (Err(l), Err(r)) => l == r,
            (Ok(l), Ok(r)) => l.total_eq(&r),
            _ => false,
        };

        assert!(is_same);

        Ok(())
    }

    #[quickcheck]
    fn function_multiple_returns(values: NonEmptyVec<LuaValue>) -> Result<(), LuaError> {
        let idents: Vec<_> = (0..values.len())
            .into_iter()
            .map(|i| format!("value{}", i))
            .map(Ident::new)
            .collect();
        let idents_str = idents.iter().join(", ");
        let module = string_parser::module(&format!(
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
        let res = module.eval(&mut context)?;
        if values.len() == 1 {
            assert!(res.total_eq(values.first()));
        } else {
            assert!(res.total_eq(&LuaValue::MultiValue(values)));
        }
        Ok(())
    }

    #[test]
    fn function_executes_side_effect() -> Result<(), LuaError> {
        let module = string_parser::module(
            "executed = nil
            function myfn() 
                executed = 1
            end
            myfn()
            return executed",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert!(res.is_truthy());
        Ok(())
    }

    #[quickcheck]
    fn local_declarations_stay_local(ident: Ident) -> Result<(), LuaError> {
        let module = string_parser::module(&format!(
            "{} = \"global\"
            function myfn()
                local {} = \"local\"
                return {}
            end
            return myfn(), {}",
            ident, ident, ident, ident
        ))?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        let expected = LuaValue::MultiValue(ne_vec![
            LuaValue::String(String::from("local")),
            LuaValue::String(String::from("global")),
        ]);
        assert_eq!(res, expected);

        Ok(())
    }
}
