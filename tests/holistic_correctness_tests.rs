use luar::{error::LuaError, lang::Eval, stdlib::std_context, syn::string_parser};

#[test]
fn heapsort() -> Result<(), LuaError> {
    let module = string_parser::module(include_str!("./heapsort.test.lua"))?;
    let mut context = std_context();
    module.eval(&mut context)?;
    Ok(())
}
