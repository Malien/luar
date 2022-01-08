use crate::{
    lang::{Eval, EvalContext, EvalError, LuaValue},
    syn::{Assignment, Statement, Var},
    util::NonEmptyVec,
};

use super::assign_to_var;

impl Eval for Statement {
    type Return = ();

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError> 
    where
        Context: EvalContext + ?Sized,
    {
        match self {
            Self::Assignment(Assignment { names, values }) => {
                // TODO: do not allocate space for values which are not going to be assigned
                let values = values
                    .into_iter()
                    .map(|expr| expr.eval(context))
                    .collect::<Result<Vec<_>, _>>()?;
                // SAFETY: Iter from which vec is collected is a NonEmptyVec
                let values = unsafe { NonEmptyVec::new_unchecked(values) };
                let (last, values) = values.pop();
                if let LuaValue::MultiValue(multi_value) = last {
                    multiple_assignment(context, names, values.into_iter().chain(multi_value));
                } else {
                    multiple_assignment(
                        context,
                        names,
                        values.into_iter().chain(std::iter::once(last)),
                    );
                }
                Ok(())
            }
            Self::FunctionCall(func_call) => func_call.eval(context).map(|_| ()),
            _ => todo!(),
        }
    }
}

fn multiple_assignment<'a, Context: EvalContext + ?Sized>(
    context: &mut Context,
    names: impl IntoIterator<Item = &'a Var>,
    values: impl Iterator<Item = LuaValue>,
) {
    for (name, value) in names.into_iter().zip(values) {
        assign_to_var(context, name, value.first_value())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use itertools::Itertools;
    use quickcheck::TestResult;

    use crate::{
        error::LuaError,
        lang::{Eval, EvalContextExt, GlobalContext, LuaFunction, LuaValue},
        lex::{Ident, Token},
        syn::{lua_parser, Module, ParseError},
        util::NonEmptyVec,
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

    #[quickcheck]
    #[allow(unstable_name_collisions)]
    fn multiple_expanded_assignment(
        idents: NonEmptyVec<Ident>,
        mut left_values: Vec<LuaValue>,
        right_values: NonEmptyVec<LuaValue>,
    ) -> Result<(), LuaError> {
        let value_idents: Vec<_> = (0..left_values.len())
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
                    .interleave_shortest(std::iter::repeat_with(|| Token::Comma)),
            )
            .chain([
                Token::Ident(Ident::new("myfn")),
                Token::OpenRoundBracket,
                Token::CloseRoundBracket,
            ])
            .collect();
        let module = lua_parser::module(&tokens)?;

        let ret_value = LuaValue::MultiValue(right_values.clone());
        let myfn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
        let mut context = GlobalContext::new();
        context.set("myfn", LuaValue::Function(myfn));
        for (ident, value) in value_idents.into_iter().zip(left_values.iter()) {
            context.set(ident, value.clone());
        }
        module.eval(&mut context)?;

        left_values.extend(right_values.into_iter());
        if idents.len() > left_values.len() {
            for ident in &idents[left_values.len()..] {
                assert_eq!(context.get(ident), &LuaValue::Nil);
            }
        }

        for (ident, value) in idents.into_iter().zip(left_values) {
            assert!(context.get(&ident).total_eq(&value));
        }

        Ok(())
    }

    fn vec_of_idents(len: usize, prefix: &str) -> Vec<Ident> {
        (0..len)
            .into_iter()
            .map(|i| format!("{}{}", prefix, i))
            .map(Ident::new)
            .collect()
    }

    #[allow(unstable_name_collisions)]
    fn multiple_expanded_assignment_in_the_middle_module(
        idents: impl Iterator<Item = Ident>,
        left_idents: impl Iterator<Item = Ident>,
        right_idents: impl Iterator<Item = Ident>,
        fn_name: Ident,
    ) -> Result<Module, ParseError> {
        let tokens: Vec<_> = idents
            .map(Token::Ident)
            .intersperse_with(|| Token::Comma)
            .chain(std::iter::once(Token::Assignment))
            .chain(
                left_idents
                    .map(Token::Ident)
                    .interleave_shortest(std::iter::repeat_with(|| Token::Comma)),
            )
            .chain([
                Token::Ident(fn_name),
                Token::OpenRoundBracket,
                Token::CloseRoundBracket,
                Token::Comma,
            ])
            .chain(
                right_idents
                    .map(Token::Ident)
                    .intersperse_with(|| Token::Comma),
            )
            .collect();
        lua_parser::module(&tokens)
    }

    #[quickcheck]
    fn multiple_expanded_assignment_in_the_middle(
        idents: NonEmptyVec<Ident>,
        left_values: Vec<LuaValue>,
        multi_value: NonEmptyVec<LuaValue>,
        right_values: NonEmptyVec<LuaValue>,
    ) -> Result<(), LuaError> {
        let left_idents = vec_of_idents(left_values.len(), "left_value");
        let right_idents = vec_of_idents(right_values.len(), "right_value");
        let module = multiple_expanded_assignment_in_the_middle_module(
            idents.iter().cloned(),
            left_idents.iter().cloned(),
            right_idents.iter().cloned(),
            Ident::new("myfn"),
        )?;

        let ret_value = LuaValue::MultiValue(multi_value.clone());
        let myfn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
        let mut context = GlobalContext::new();
        context.set("myfn", LuaValue::Function(myfn));
        for (ident, value) in left_idents.into_iter().zip(left_values.iter()) {
            context.set(ident, value.clone());
        }
        for (ident, value) in right_idents.into_iter().zip(right_values.iter()) {
            context.set(ident, value.clone());
        }
        module.eval(&mut context)?;

        let resulting_values: Vec<LuaValue> = left_values
            .into_iter()
            .chain(std::iter::once(multi_value.move_first()))
            .chain(right_values)
            .collect();

        if idents.len() > resulting_values.len() {
            for ident in &idents[resulting_values.len()..] {
                assert_eq!(context.get(ident), &LuaValue::Nil);
            }
        }

        for (ident, value) in idents.into_iter().zip(resulting_values) {
            assert!(context.get(&ident).total_eq(&value));
        }

        Ok(())
    }
}
