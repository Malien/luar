use crate::lang::{EvalError, GlobalContext, LuaFunction, LuaValue, ReturnValue};

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
}

fn define_fn(
    ctx: &mut GlobalContext,
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
    ctx: &mut GlobalContext,
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
    use test_util::run_lua_test;

    use crate::error::LuaError;

    use super::std_context;

    #[test]
    fn lua_test() -> Result<(), LuaError> {
        run_lua_test!("./stdlib.test.lua", std_context())
    }
}
