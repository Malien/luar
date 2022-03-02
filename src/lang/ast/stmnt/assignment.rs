use crate::{
    lang::{ast::assign_to_var, Eval, EvalContext, EvalError, LuaValue},
    syn::{Assignment, Expression, Var},
};

impl Eval for Assignment {
    type Return = ();

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        let Assignment { names, values } = self;
        assignment_values(context, values).map(|values| multiple_assignment(context, names, values))
    }
}

pub struct TailValuesIter<I> {
    values: I,
    hold: Option<LuaValue>,
    tail: Option<std::vec::IntoIter<LuaValue>>,
}

impl<I> Iterator for TailValuesIter<I>
where
    I: Iterator<Item = LuaValue>,
{
    type Item = LuaValue;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.values.next(), &mut self.hold, &mut self.tail) {
                (Some(next), Some(hold), _) => return Some(std::mem::replace(hold, next)),
                (Some(next), hold, _) => *hold = Some(next),
                (None, _, Some(tail)) => return tail.next(),
                (None, hold, tail) => {
                    let hold_value = hold.take();
                    if let Some(LuaValue::MultiValue(values)) = hold_value {
                        *tail = Some(values.into_iter());
                    } else {
                        return hold_value;
                    }
                }
            }
        }
    }
}

pub fn assignment_values<'a, Context: EvalContext + ?Sized>(
    context: &mut Context,
    values: impl IntoIterator<Item = &'a Expression>,
) -> Result<impl Iterator<Item = LuaValue>, EvalError> {
    values
        .into_iter()
        .map(|expr| expr.eval(context))
        .collect::<Result<Vec<_>, _>>()
        .map(|values| tail_values(values).chain(std::iter::repeat_with(|| LuaValue::Nil)))
}

pub fn tail_values<I>(values: I) -> TailValuesIter<I::IntoIter>
where
    I: IntoIterator<Item = LuaValue>,
{
    TailValuesIter {
        values: values.into_iter(),
        hold: None,
        tail: None,
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
        lang::{Eval, EvalContext, EvalContextExt, GlobalContext, LuaFunction, LuaValue},
        lex::{Ident, Token},
        ne_vec,
        syn::{lua_parser, Module, ParseError},
        util::NonEmptyVec,
    };

    use super::tail_values;

    #[test]
    fn tail_values_empty() {
        let values = std::iter::empty();
        let mut tail_values = tail_values(values);
        assert!(tail_values.next().is_none());
    }

    #[test]
    fn tail_values_iter_simple_values() {
        let values = vec![
            LuaValue::Nil,
            LuaValue::number(42),
            LuaValue::String("hello".into()),
        ];
        let tail_values: Vec<_> = tail_values(values.clone()).collect();
        assert_eq!(values, tail_values);
    }

    #[test]
    fn tail_values_expand_end_multi_value() {
        let multi_value = LuaValue::MultiValue(ne_vec![
            LuaValue::Nil,
            LuaValue::String("hello".into()),
            LuaValue::number(69),
        ]);
        let values = vec![LuaValue::Nil, LuaValue::number(42), multi_value];
        let tail_values: Vec<_> = tail_values(values).collect();
        let expected = vec![
            LuaValue::Nil,
            LuaValue::number(42),
            LuaValue::Nil,
            LuaValue::String("hello".into()),
            LuaValue::number(69),
        ];
        assert_eq!(tail_values, expected);
    }

    #[test]
    fn tail_values_does_not_expand_multi_value_in_the_middle() {
        let multi_value = LuaValue::MultiValue(ne_vec![
            LuaValue::Nil,
            LuaValue::String("hello".into()),
            LuaValue::number(69),
        ]);
        let values = vec![LuaValue::Nil, multi_value, LuaValue::number(42)];
        let tail_values: Vec<_> = tail_values(values.clone()).collect();
        assert_eq!(tail_values, values);
    }

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

    #[allow(unstable_name_collisions)]
    fn multiple_assignment_tokens(
        idents: impl Iterator<Item = Ident>,
        value_idents: impl Iterator<Item = Ident>,
    ) -> impl Iterator<Item = Token> {
        idents
            .map(Token::Ident)
            .intersperse_with(|| Token::Comma)
            .chain(std::iter::once(Token::Assignment))
            .chain(
                value_idents
                    .map(Token::Ident)
                    .intersperse_with(|| Token::Comma),
            )
    }

    fn multi_return_fn(ret: NonEmptyVec<LuaValue>) -> LuaValue {
        let ret_value = LuaValue::MultiValue(ret);
        let lua_fn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
        LuaValue::Function(lua_fn)
    }

    fn assign_values(
        context: &mut impl EvalContext,
        names: impl IntoIterator<Item = Ident>,
        values: impl IntoIterator<Item = LuaValue>,
    ) {
        for (name, value) in names.into_iter().zip(values) {
            context.set(name, value)
        }
    }

    fn assert_multiple_assignment(
        context: &impl EvalContext,
        idents: Vec<Ident>,
        values: Vec<LuaValue>,
    ) {
        if idents.len() > values.len() {
            for ident in &idents[values.len()..] {
                assert_eq!(context.get(ident), &LuaValue::Nil);
            }
        }

        for (ident, value) in idents.into_iter().zip(values) {
            let res = context.get(&ident).total_eq(&value);
            if !res {
                println!("Nope! {}\t{}", context.get(&ident), value);
            }
            assert!(res);
        }
    }

    fn put_dummy_values<'a>(
        context: &mut impl EvalContext,
        idents: impl IntoIterator<Item = &'a Ident>,
    ) {
        for ident in idents {
            context.set(ident.clone(), LuaValue::number(42));
        }
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
        let value_idents = vec_of_idents(values.len(), "value");

        let tokens: Vec<_> =
            multiple_assignment_tokens(idents.iter().cloned(), value_idents.iter().cloned())
                .collect();
        let module = lua_parser::module(&tokens)?;

        let mut context = GlobalContext::new();
        assign_values(
            &mut context,
            value_idents.iter().cloned(),
            values.iter().cloned(),
        );
        put_dummy_values(&mut context, &idents);

        module.eval(&mut context)?;
        assert_multiple_assignment(&context, idents, values.into());

        Ok(TestResult::passed())
    }

    #[quickcheck]
    #[allow(unstable_name_collisions)]
    fn multiple_expanded_assignment(
        idents: HashSet<Ident>,
        mut left_values: Vec<LuaValue>,
        right_values: NonEmptyVec<LuaValue>,
    ) -> Result<TestResult, LuaError> {
        if idents.len() == 0 {
            return Ok(TestResult::discard());
        }
        // Make iteration order deterministic
        let idents: Vec<_> = idents.into_iter().collect();
        let value_idents = vec_of_idents(left_values.len(), "value");

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

        let mut context = GlobalContext::new();
        context.set("myfn", multi_return_fn(right_values.clone()));
        assign_values(&mut context, value_idents, left_values.iter().cloned());
        put_dummy_values(&mut context, &idents);
        module.eval(&mut context)?;

        left_values.extend(right_values.into_iter());
        assert_multiple_assignment(&mut context, idents.into(), left_values);

        Ok(TestResult::passed())
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
        idents: HashSet<Ident>,
        left_values: Vec<LuaValue>,
        multi_value: NonEmptyVec<LuaValue>,
        right_values: NonEmptyVec<LuaValue>,
    ) -> Result<TestResult, LuaError> {
        if idents.len() == 0 {
            return Ok(TestResult::discard());
        }
        // Make iteration order deterministic
        let idents: Vec<_> = idents.into_iter().collect();
        let left_idents = vec_of_idents(left_values.len(), "left_value");
        let right_idents = vec_of_idents(right_values.len(), "right_value");

        let module = multiple_expanded_assignment_in_the_middle_module(
            idents.iter().cloned(),
            left_idents.iter().cloned(),
            right_idents.iter().cloned(),
            Ident::new("myfn"),
        )?;

        let mut context = GlobalContext::new();
        context.set("myfn", multi_return_fn(multi_value.clone()));
        assign_values(&mut context, left_idents, left_values.iter().cloned());
        assign_values(&mut context, right_idents, right_values.iter().cloned());
        put_dummy_values(&mut context, &idents);

        module.eval(&mut context)?;

        let resulting_values: Vec<LuaValue> = left_values
            .into_iter()
            .chain(std::iter::once(multi_value.move_first()))
            .chain(right_values)
            .collect();
        assert_multiple_assignment(&context, idents, resulting_values);

        Ok(TestResult::passed())
    }
}
