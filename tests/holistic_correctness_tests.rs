use luar::{error::LuaError, stdlib::std_context, syn::lua_parser, lang::Eval};

#[test]
fn heapsort() -> Result<(), LuaError> {
    let module = lua_parser::module(include_str!("./heapsort.test.lua"))?;
    let mut context = std_context();
    module.eval(&mut context)?;
    Ok(())
}
