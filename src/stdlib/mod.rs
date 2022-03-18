use crate::lang::{
    EvalContext, EvalContextExt, EvalError, GlobalContext, LuaFunction, LuaValue, ReturnValue,
};

pub mod fns;

pub fn std_context() -> GlobalContext {
    let mut ctx = GlobalContext::new();
    define_std_lib(&mut ctx);
    return ctx;
}

pub fn define_std_lib(ctx: &mut impl EvalContext) {
    define_total_fn(ctx, "tonumber", fns::tonumber);
    define_fn(ctx, "print", fns::print_stdout);
    define_total_fn(ctx, "random", fns::random);
}

fn define_fn(
    ctx: &mut impl EvalContext,
    name: &str,
    fun: impl Fn(&[LuaValue]) -> Result<LuaValue, EvalError> + 'static,
) {
    ctx.set(
        name,
        LuaValue::Function(LuaFunction::new(move |_, args| {
            fun(args).map(ReturnValue::from)
        })),
    )
}

fn define_total_fn(
    ctx: &mut impl EvalContext,
    name: &str,
    fun: impl Fn(&[LuaValue]) -> LuaValue + 'static,
) {
    ctx.set(
        name,
        LuaValue::Function(LuaFunction::new(move |_, args| Ok(fun(args).into()))),
    );
}

#[cfg(test)]
mod test {
    use crate::error::LuaError;
    use crate::run_lua_test;

    use super::std_context;

    #[test]
    fn lua_test() -> Result<(), LuaError> {
        run_lua_test!("./stdlib.test.lua", std_context())
    }
}
