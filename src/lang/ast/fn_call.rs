use crate::{
    lang::{Eval, EvalContext, EvalError, LuaValue, TypeError},
    syn::{FunctionCall, FunctionCallArgs},
};

impl Eval for FunctionCall {
    type Return = LuaValue;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self {
            Self::Function { func, args } => args.eval(context).and_then(|args| {
                let fn_value = func.eval(context)?;
                call_value(context, &fn_value, &args)
            }),
            Self::Method { .. } => todo!(),
        }
    }
}

fn call_value(
    context: &mut impl EvalContext,
    func: &LuaValue,
    args: &[LuaValue],
) -> Result<LuaValue, EvalError> {
    if let LuaValue::Function(func) = func {
        func.call(context, args)
    } else {
        Err(EvalError::TypeError(TypeError::IsNotCallable(func.clone())))
    }
}

impl Eval for FunctionCallArgs {
    type Return = Vec<LuaValue>;

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self {
            Self::Arglist(exprs) => exprs.into_iter().map(|expr| expr.eval(context)).collect(),
            Self::Table(table) => table.eval(context).map(|table| vec![table]),
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;

    use quickcheck::TestResult;

    use super::Eval;
    use crate::error::LuaError;
    use crate::lang::{EvalContextExt, EvalError, GlobalContext, LuaFunction, LuaValue, TypeError};
    use crate::syn;
    use crate::util::NonEmptyVec;

    #[test]
    fn eval_fn_call() -> Result<(), LuaError> {
        let module = syn::string_parser::module("myfn()")?;
        let called = Rc::new(RefCell::new(false));
        let myfn = LuaFunction::new({
            let called = Rc::clone(&called);
            move |_, _| {
                let mut called = called.borrow_mut();
                *called = true;
                Ok(LuaValue::Nil)
            }
        });
        let mut context = GlobalContext::new();
        context.set("myfn", LuaValue::Function(myfn));
        module.eval(&mut context)?;
        let called = called.borrow();
        assert!(*called);
        Ok(())
    }

    #[quickcheck]
    fn eval_fn_return(ret_value: LuaValue) -> Result<(), LuaError> {
        let module = syn::string_parser::module("return myfn()")?;
        let mut context = GlobalContext::new();
        let myfn = LuaFunction::new({
            let ret_value = ret_value.clone();
            move |_, _| Ok(ret_value.clone())
        });
        context.set("myfn", LuaValue::Function(myfn));
        let res = module.eval(&mut context)?;
        assert!(ret_value.total_eq(&res));
        Ok(())
    }

    #[quickcheck]
    fn calling_not_a_function_value_is_an_error(value: LuaValue) -> Result<TestResult, LuaError> {
        if value.is_function() {
            return Ok(TestResult::discard());
        }

        let module = syn::string_parser::module("value()")?;
        let mut context = GlobalContext::new();
        context.set("value", value);
        let res = module.eval(&mut context);
        assert!(matches!(
            res,
            Err(EvalError::TypeError(TypeError::IsNotCallable(_)))
        ));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_fn_call_multiple_returns(values: NonEmptyVec<LuaValue>) -> Result<(), LuaError> {
        let module = syn::string_parser::module("return myfn()")?;
        let mut context = GlobalContext::new();
        let ret_values = LuaValue::MultiValue(values);
        let myfn = LuaFunction::new({
            let ret_values = ret_values.clone();
            move |_, _| Ok(ret_values.clone())
        });
        context.set("myfn", LuaValue::Function(myfn));
        let res = module.eval(&mut context)?;
        assert!(ret_values.total_eq(&res));
        Ok(())
    }
}
