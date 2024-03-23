use crate::{
    lang::{GlobalContext, LuaValue, ReturnValue},
    EvalError,
};

pub mod fns;

pub fn std_context() -> GlobalContext {
    let mut ctx = GlobalContext::new();
    define_std_lib(&mut ctx);
    return ctx;
}

pub(crate) fn define_std_lib(ctx: &mut GlobalContext) {
    define_total_fn(ctx, "tonumber", fns::tonumber);
    define_fn(ctx, "print", fns::print_stdout);
    define_total_fn(ctx, "random", fns::random);
    define_fn(ctx, "floor", fns::floor);
    define_fn(ctx, "assert", fns::assert);
    define_fn(ctx, "strlen", fns::strlen);
    define_fn(ctx, "strsub", fns::strsub);
    define_total_fn(ctx, "type", fns::lua_type);
}

fn define_fn(
    ctx: &mut GlobalContext,
    name: &str,
    fun: impl Fn(&[LuaValue]) -> Result<LuaValue, EvalError> + 'static,
) {
    ctx.set(
        name,
        LuaValue::function(move |_, args| fun(args).map(ReturnValue::from)),
    )
}

fn define_total_fn(
    ctx: &mut GlobalContext,
    name: &str,
    fun: impl Fn(&[LuaValue]) -> LuaValue + 'static,
) {
    ctx.set(
        name,
        LuaValue::function(move |_, args| Ok(fun(args).into())),
    );
}

#[cfg(test)]
mod test {
    use crate::LuaError;
    use luar_syn::lua_parser;

    use super::std_context;

    #[test]
    fn lua_test() -> Result<(), LuaError> {
        let mut context = std_context();
        let test_module = lua_parser::module(include_str!("./stdlib.test.lua"))?;
        crate::eval_module(&test_module, &mut context)?;
        Ok(())
    }
}
