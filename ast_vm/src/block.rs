use crate::{
    lang::{LocalScope, ScopeHolder},
    EvalError,
};
use luar_syn::Block;

use super::{eval_ret, eval_stmnt, ControlFlow};

pub(crate) fn eval_block(
    block: &Block,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ControlFlow, EvalError> {
    for statement in &block.statements {
        if let ControlFlow::Return(value) = eval_stmnt(statement, scope)? {
            return Ok(ControlFlow::Return(value));
        }
    }
    block
        .ret
        .as_ref()
        .map(|ret| eval_ret(ret, scope).map(ControlFlow::Return))
        .unwrap_or(Ok(ControlFlow::Continue))
}

#[cfg(test)]
mod test {
    use crate as ast_vm;
    use crate::{lang::Context, LuaError};
    use luar_syn::lua_parser;

    #[test]
    fn early_returns_from_blocks_stop_flow_of_execution() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "if 1 then
                return 1
            end
            return nil",
        )?;
        let mut context = Context::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.assert_single().is_truthy());
        Ok(())
    }

    #[test]
    fn early_returns_from_functions_stop_flow_of_execution() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function fn()
                if 1 then
                    return 1
                end
                return nil
            end
            return fn()",
        )?;
        let mut context = Context::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.assert_single().is_truthy());
        Ok(())
    }
}
