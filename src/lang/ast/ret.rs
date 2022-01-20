use crate::{
    lang::{Eval, EvalContext, EvalError, LuaValue},
    syn::Return,
    util::NonEmptyVec,
};

impl Eval for Return {
    type Return = LuaValue;

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
                // SAFETY: values is produced from NonEmpty vec, so values are not empty as well
                .map(|values| LuaValue::MultiValue(unsafe { NonEmptyVec::new_unchecked(values) }))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::LuaError,
        lang::{Eval, EvalContextExt, GlobalContext, LuaValue},
        lex::Ident,
        syn::{Expression, Module, Return, Var},
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
            assert!(res.total_eq(values.first()));
        } else {
            assert!(res.total_eq(&LuaValue::MultiValue(values)));
        }
        Ok(())
    }
}
