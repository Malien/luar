use crate::{
    lang::{LocalScope, ReturnValue, ScopeHolder},
    EvalError,
};
use luar_syn::Return;

use super::{eval_expr, tail_values};

pub(crate) fn eval_ret(
    ret: &Return,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<ReturnValue, EvalError> {
    if ret.0.len() <= 1 {
        match ret.0.first() {
            Some(expr) => eval_expr(expr, scope),
            None => Ok(ReturnValue::NIL),
        }
    } else {
        ret.0
            .iter()
            .map(|expr| eval_expr(expr, scope))
            .collect::<Result<Vec<_>, _>>()
            .map(tail_values)
            .map(Iterator::collect)
            .map(ReturnValue)
    }
}

#[cfg(test)]
mod test {
    use crate as ast_vm;
    use crate::{
        lang::{GlobalContext, LuaValue, ReturnValue},
        LuaError,
    };
    use luar_lex::Ident;
    use luar_syn::{Expression, FunctionCall, FunctionCallArgs, Module, Return, Var};
    use non_empty::NonEmptyVec;

    #[quickcheck]
    fn eval_multiple_return(values: NonEmptyVec<LuaValue>) -> Result<(), LuaError> {
        let idents: Vec<_> = (0..values.len().get())
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
        let expected: ReturnValue = values.into_iter().collect();
        assert!(res.total_eq(&expected));
        Ok(())
    }

    #[quickcheck]
    fn eval_multiple_concatenated_return(
        v1: NonEmptyVec<LuaValue>,
        v2: NonEmptyVec<LuaValue>,
    ) -> Result<(), LuaError> {
        let idents: Vec<_> = (0..v1.len().get())
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

        let ret_value: ReturnValue = v2.iter().cloned().collect();
        let mult_fn = LuaValue::function(move |_, _| Ok(ret_value.clone()));
        context.set("mult", mult_fn);

        for (val, ident) in v1.iter().zip(idents) {
            context.set(ident, val.clone());
        }
        let res = ast_vm::eval_module(&module, &mut context)?;
        let combined: ReturnValue = v1.into_iter().chain(v2).collect();
        assert!(res.total_eq(&combined));
        Ok(())
    }
}
