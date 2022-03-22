use luar::{ast_vm, error::LuaError, stdlib::std_context, syn::lua_parser};

#[test]
fn heapsort() -> Result<(), LuaError> {
    let module = lua_parser::module(include_str!("./heapsort.test.lua"))?;
    let mut context = std_context();
    ast_vm::eval_module(&module, &mut context)?;
    Ok(())
}
