use crate::{
    lang::{Eval, EvalContext, EvalError, LuaValue, EvalContextExt},
    syn::{Assignment, Statement, Var},
};

impl Eval for Statement {
    type Return = ();

    fn eval(&self, context: &mut impl EvalContext) -> Result<Self::Return, EvalError> {
        match self {
            Self::Assignment(Assignment { names, values }) => {
                let (stored, throwaway) = values.split_at(names.len());
                let stored = stored
                    .into_iter()
                    .map(|expr| expr.eval(context))
                    .collect::<Result<Vec<_>, _>>()?;
                for expr in throwaway {
                    expr.eval(context)?;
                }
                for (var, value) in names.into_iter().zip(stored) {
                    assign_to_var(context, var, value);
                }
                Ok(())
            }
            Self::FunctionCall(func_call) => func_call.eval(context).map(|_| ()),
            _ => todo!(),
        }
    }
}

fn assign_to_var(context: &mut impl EvalContext, var: &Var, value: LuaValue) {
    match var {
        Var::Named(ident) => context.set(ident.clone(), value),
        _ => todo!(),
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use itertools::Itertools;
    use quickcheck::TestResult;

    use crate::{
        error::LuaError,
        lang::{GlobalContext, LuaValue, EvalContextExt, Eval},
        lex::{Ident, Token},
        syn::lua_parser, util::NonEmptyVec,
    };

    #[quickcheck]
    fn eval_single_assignment(ident: Ident, v1: LuaValue, v2: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module(&[
            Token::Ident(ident.clone()),
            Token::Assignment,
            Token::Ident(Ident::new("value")),
        ])?;
        let mut context = GlobalContext::new();
        assert_eq!(context.get(&ident), &LuaValue::Nil);
        context.set("value", v1.clone());
        module.eval(&mut context)?;
        assert!(context.get(&ident).total_eq(&v1));
        context.set("value", v2.clone());
        module.eval(&mut context)?;
        assert!(context.get(&ident).total_eq(&v2));
        Ok(())
    }

    #[quickcheck]
    #[allow(unstable_name_collisions)]
    fn eval_multiple_assignment(
        idents: HashSet<Ident>,
        values: NonEmptyVec<LuaValue>,
    ) -> Result<TestResult, LuaError> {
        if idents.len() == 0 {
            return Ok(TestResult::discard());
        }
        // Make iteration order deterministic
        let idents: Vec<_> = idents.into_iter().collect();
        let value_idents: Vec<_> = (0..values.len())
            .into_iter()
            .map(|i| format!("value{}", i))
            .map(Ident::new)
            .collect();
        let tokens: Vec<_> = idents
            .iter()
            .cloned()
            .map(Token::Ident)
            .intersperse_with(|| Token::Comma)
            .chain(std::iter::once(Token::Assignment))
            .chain(
                value_idents
                    .iter()
                    .cloned()
                    .map(Token::Ident)
                    .intersperse_with(|| Token::Comma),
            )
            .collect();
        let module = lua_parser::module(&tokens)?;
        let mut context = GlobalContext::new();
        for (ident, value) in value_idents.iter().cloned().zip(values.iter().cloned()) {
            context.set(ident, value);
        }
        for ident in &idents {
            context.set(ident.clone(), LuaValue::Number(42f64));
        }
        module.eval(&mut context)?;

        if idents.len() > values.len() {
            for ident in &idents[values.len()..] {
                assert_eq!(context.get(ident), &LuaValue::Nil);
            }
        }

        for (ident, value) in idents.into_iter().zip(values) {
            assert!(context.get(&ident).total_eq(&value));
        }

        Ok(TestResult::passed())
    }
}
