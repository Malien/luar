use crate::{
    ast_vm::{eval_block, eval_expr, ControlFlow},
    lang::{LocalScope, ScopeHolder},
};
use luar_syn::WhileLoop;

pub(crate) fn eval_while_loop(
    while_loop: &WhileLoop,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ControlFlow, crate::lang::EvalError> {
    let WhileLoop { condition, body } = while_loop;
    while eval_expr(condition, scope)?.first_value().is_truthy() {
        if let ControlFlow::Return(ret_value) = eval_block(body, &mut scope.child_scope())? {
            return Ok(ControlFlow::Return(ret_value));
        }
    }
    Ok(ControlFlow::Continue)
}

#[cfg(test)]
mod test {
    use crate::{
        ast_vm,
        error::LuaError,
        lang::{GlobalContext, LuaValue, ReturnValue},
    };
    use luar_syn::lua_parser;
    use non_empty::ne_vec;

    #[test]
    fn while_loop_with_falsy_condition_does_not_execute_body() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "while nil do
                side_effect_committed = 1
            end
            return side_effect_committed",
        )?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.assert_single().is_falsy());
        Ok(())
    }

    #[test]
    fn while_loop_with_truthy_condition_executes_body_at_least_vm_once() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "while not side_effect_committed do
                side_effect_committed = 1
            end
            return side_effect_committed",
        )?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.assert_single().is_truthy());
        Ok(())
    }

    #[quickcheck]
    fn while_loop_executes_until_condition_is_true(times: u8) -> Result<(), LuaError> {
        let module = lua_parser::module(
            "count_executed = 0
            while i ~= 0 do
                count_executed = count_executed + 1
                i = i - 1
            end
            return i, count_executed",
        )?;
        let mut context = GlobalContext::new();
        context.set("i", LuaValue::number(times));
        let res = ast_vm::eval_module(&module, &mut context)?;
        let expected =
            ReturnValue::MultiValue(ne_vec![LuaValue::number(0i32), LuaValue::number(times)]);
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn while_loop_early_return() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "while 1 do
                return 1
            end",
        )?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.assert_single().is_truthy());
        Ok(())
    }
}
