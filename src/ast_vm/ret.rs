use non_empty::NonEmptyVec;

use crate::{
    lang::{EvalError, LocalScope, ReturnValue, ScopeHolder},
    syn::Return,
};

use super::{eval_expr, tail_values};

pub(crate) fn eval_ret(
    ret: &Return,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ReturnValue, EvalError> {
    if ret.0.len() <= 1 {
        match ret.0.first() {
            Some(expr) => eval_expr(expr, scope),
            None => Ok(ReturnValue::Nil),
        }
    } else {
        ret.0
            .iter()
            .map(|expr| eval_expr(expr, scope))
            .collect::<Result<Vec<_>, _>>()
            .map(tail_values)
            .map(Iterator::collect)
            // SAFETY: values is produced from NonEmpty vec, so values are not empty as well
            .map(|values| unsafe { NonEmptyVec::new_unchecked(values) })
            .map(ReturnValue::MultiValue)
    }
}

#[cfg(test)]
mod test {
    use non_empty::NonEmptyVec;

    use crate::{
        ast_vm,
        error::LuaError,
        lang::{GlobalContext, LuaFunction, LuaValue, ReturnValue},
        lex::Ident,
        syn::{Expression, FunctionCall, FunctionCallArgs, Module, Return, Var},
    };

    #[quickcheck]
    fn eval_multiple_return(values: NonEmptyVec<LuaValue>) -> Result<(), LuaError> {
        let idents: Vec<_> = (0..values.len())
            .into_iter()
            .map(|i| format!("value{}", i))
            .map(Ident::new)
            .collect();
        let module = Module {
            chunks: vec![],
            ret: Some(Return(
                idents
                    .iter()
                    .cloned()
                    .map(Var::Named)
                    .map(Expression::Variable)
                    .collect(),
            )),
        };
        let mut context = GlobalContext::new();
        for (val, ident) in values.iter().zip(idents) {
            context.set(ident, val.clone());
        }
        let res = ast_vm::eval_module(&module, &mut context)?;
        if values.len() == 1 {
            assert!(res.total_eq(&values.move_first().into()));
        } else {
            assert!(res.total_eq(&ReturnValue::MultiValue(values)));
        }
        Ok(())
    }

    #[quickcheck]
    fn eval_multiple_concatenated_return(
        v1: NonEmptyVec<LuaValue>,
        v2: NonEmptyVec<LuaValue>,
    ) -> Result<(), LuaError> {
        let idents: Vec<_> = (0..v1.len())
            .into_iter()
            .map(|i| format!("value{}", i))
            .map(Ident::new)
            .collect();
        let module = Module {
            chunks: vec![],
            ret: Some(Return(
                idents
                    .iter()
                    .cloned()
                    .map(Var::Named)
                    .map(Expression::Variable)
                    .chain(std::iter::once(Expression::FunctionCall(
                        FunctionCall::Function {
                            func: Var::Named(Ident::new("mult")),
                            args: FunctionCallArgs::Arglist(vec![]),
                        },
                    )))
                    .collect(),
            )),
        };
        let mut context = GlobalContext::new();

        let ret_value = ReturnValue::MultiValue(v2.clone());
        let mult_fn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
        context.set("mult", LuaValue::Function(mult_fn));

        for (val, ident) in v1.iter().zip(idents) {
            context.set(ident, val.clone());
        }
        let res = ast_vm::eval_module(&module, &mut context)?;
        let combined = NonEmptyVec::try_new(v1.into_iter().chain(v2).collect()).unwrap();
        assert!(res.total_eq(&ReturnValue::MultiValue(combined)));
        Ok(())
    }
}
