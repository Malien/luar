use crate::{
    lang::{Context, LocalScope, LuaValue, ReturnValue, ScopeHolder, TableRef}, opt::call_function, EvalError, TypeError
};
use luar_syn::{FunctionCall, FunctionCallArgs};

use super::{eval_expr, eval_tbl_constructor, eval_var};

pub(crate) fn eval_fn_call(
    call: &FunctionCall,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ReturnValue, EvalError> {
    match call {
        FunctionCall::Function { func, args } => eval_fn_args(args, scope).and_then(|args| {
            let fn_value = eval_var(func, scope)?;
            call_value(scope.global_mut(), &fn_value, &args)
        }),
        FunctionCall::Method { func, args, method } => {
            todo!("Cannot evaluate method call {func}:{method}{args} yet")
        }
    }
}

pub(crate) fn call_value(
    context: &mut Context,
    func: &LuaValue,
    args: &[LuaValue],
) -> Result<ReturnValue, EvalError> {
    match func {
        LuaValue::Function(func) => call_function(func, context, args),
        LuaValue::NativeFunction(func) => func.call(context, args),
        _ => Err(EvalError::from(TypeError::IsNotCallable(func.clone()))),
    }
}

fn eval_fn_args(
    args: &FunctionCallArgs,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<Vec<LuaValue>, EvalError> {
    match args {
        FunctionCallArgs::Arglist(exprs) => exprs
            .into_iter()
            .map(|expr| eval_expr(expr, scope))
            .map(|arg| arg.map(ReturnValue::first_value))
            .collect(),
        FunctionCallArgs::Table(table) => eval_tbl_constructor(table, scope)
            .map(TableRef::from)
            .map(LuaValue::Table)
            .map(|table| vec![table]),
    }
}

#[cfg(test)]
mod test {
    use crate as ast_vm;
    use crate::{
        lang::{Context, LuaValue, ReturnValue},
        LuaError, TypeError,
    };
    use luar_error::assert_type_error;
    use luar_syn::lua_parser;
    use non_empty::NonEmptyVec;
    use quickcheck::TestResult;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn eval_fn_call() -> Result<(), LuaError> {
        let module = lua_parser::module("myfn()")?;
        let called = Rc::new(RefCell::new(false));
        let myfn = LuaValue::function({
            let called = Rc::clone(&called);
            move |_, _| {
                let mut called = called.borrow_mut();
                *called = true;
                Ok(ReturnValue::NIL)
            }
        });
        let mut context = Context::new();
        context.set("myfn", myfn);
        ast_vm::eval_module(&module, &mut context)?;
        let called = called.borrow();
        assert!(*called);
        Ok(())
    }

    #[quickcheck]
    fn eval_fn_return(ret_value: LuaValue) -> Result<(), LuaError> {
        let ret_value = ReturnValue::from(ret_value);
        let module = lua_parser::module("return myfn()")?;
        let mut context = Context::new();
        let myfn = LuaValue::function({
            let ret_value = ret_value.clone();
            move |_, _| Ok(ret_value.clone())
        });
        context.set("myfn", myfn);
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(ret_value.total_eq(&res));
        Ok(())
    }

    #[quickcheck]
    fn calling_not_a_function_value_is_an_error(value: LuaValue) -> Result<TestResult, LuaError> {
        if value.is_callable() {
            return Ok(TestResult::discard());
        }

        let module = lua_parser::module("value()")?;
        let mut context = Context::new();
        context.set("value", value);
        let res = ast_vm::eval_module(&module, &mut context);
        assert_type_error!(TypeError::IsNotCallable(_), res);
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_fn_call_multiple_returns(values: NonEmptyVec<LuaValue>) -> Result<(), LuaError> {
        let module = lua_parser::module("return myfn()")?;
        let mut context = Context::new();
        let ret_values: ReturnValue = values.into_iter().collect();
        let myfn = LuaValue::function({
            let ret_values = ret_values.clone();
            move |_, _| Ok(ret_values.clone())
        });
        context.set("myfn", myfn);
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(ret_values.total_eq(&res));
        Ok(())
    }
}
