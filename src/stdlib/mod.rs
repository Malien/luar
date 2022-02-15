use crate::lang::{EvalContext, EvalContextExt, EvalError, GlobalContext, LuaFunction, LuaValue};

pub mod fns;

pub fn std_context() -> GlobalContext {
    let mut ctx = GlobalContext::new();
    define_std_lib(&mut ctx);
    return ctx;
}

pub fn define_std_lib(ctx: &mut impl EvalContext) {
    define_total_fn(ctx, "tonumber", fns::tonumber);
    define_fn(ctx, "print", fns::print_stdout);
}

fn define_fn(
    ctx: &mut impl EvalContext,
    name: &str,
    fun: impl Fn(&[LuaValue]) -> Result<LuaValue, EvalError> + 'static,
) {
    ctx.set(
        name,
        LuaValue::Function(LuaFunction::new(move |_, args| fun(args))),
    )
}

fn define_total_fn(
    ctx: &mut impl EvalContext,
    name: &str,
    fun: impl Fn(&[LuaValue]) -> LuaValue + 'static,
) {
    ctx.set(
        name,
        LuaValue::Function(LuaFunction::new(move |_, args| Ok(fun(args)))),
    );
}

#[cfg(test)]
mod test {
    use crate::error::LuaError;
    use crate::lang::{Eval, EvalContext, EvalContextExt, EvalError, LuaFunction, LuaValue};
    use crate::syn;

    use super::std_context;

    fn lua_assert(_: &mut dyn EvalContext, args: &[LuaValue]) -> Result<LuaValue, EvalError> {
        match args.first() {
            None | Some(LuaValue::Nil) => Err(EvalError::AssertionError),
            _ => Ok(LuaValue::Nil),
        }
    }

    #[test]
    fn lua_test() -> Result<(), LuaError> {
        let mut context = std_context();
        context.set("assert", LuaValue::Function(LuaFunction::new(lua_assert)));
        let test_module = syn::string_parser::module(include_str!("./stdlib_test.lua"))?;
        test_module.eval(&mut context)?;
        Ok(())
    }
}
