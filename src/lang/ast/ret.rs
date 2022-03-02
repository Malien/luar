use crate::{
    lang::{Eval, EvalContext, EvalError, ReturnValue},
    syn::Return,
    util::NonEmptyVec,
};

use super::tail_values;

impl Eval for Return {
    type Return = ReturnValue;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        if self.0.len() == 1 {
            self.0.first().eval(context)
        } else {
            self.0
                .iter()
                .map(|expr| expr.eval(context))
                .collect::<Result<Vec<_>, _>>()
                .map(tail_values)
                .map(Iterator::collect)
                // SAFETY: values is produced from NonEmpty vec, so values are not empty as well
                .map(|values| unsafe { NonEmptyVec::new_unchecked(values) })
                .map(ReturnValue::MultiValue)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::LuaError,
        lang::{Eval, EvalContextExt, GlobalContext, LuaFunction, LuaValue, ReturnValue},
        lex::Ident,
        syn::{Expression, FunctionCall, FunctionCallArgs, Module, Return, Var},
        util::NonEmptyVec,
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
            ret: Some(Return(NonEmptyVec::new(
                idents
                    .iter()
                    .cloned()
                    .map(Var::Named)
                    .map(Expression::Variable)
                    .collect(),
            ))),
        };
        let mut context = GlobalContext::new();
        for (val, ident) in values.iter().zip(idents) {
            context.set(ident, val.clone());
        }
        let res = module.eval(&mut context)?;
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
            ret: Some(Return(NonEmptyVec::new(
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
            ))),
        };
        let mut context = GlobalContext::new();

        let ret_value = ReturnValue::MultiValue(v2.clone());
        let mult_fn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
        context.set("mult", LuaValue::Function(mult_fn));

        for (val, ident) in v1.iter().zip(idents) {
            context.set(ident, val.clone());
        }
        let res = module.eval(&mut context)?;
        let combined = NonEmptyVec::try_new(v1.into_iter().chain(v2).collect()).unwrap();
        assert!(res.total_eq(&ReturnValue::MultiValue(combined)));
        Ok(())
    }
}
