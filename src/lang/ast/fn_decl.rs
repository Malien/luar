use crate::{
    lang::{Eval, LuaFunction, LuaValue, EvalContext},
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
                    move |context, _| body.eval(context)
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
    use crate::{
        error::LuaError,
        lang::{Eval, EvalContextExt, GlobalContext, LuaValue},
        lex::Ident,
        syn::string_parser,
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
}
