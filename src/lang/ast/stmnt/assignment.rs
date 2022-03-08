use crate::{
    lang::{
        ast::{assign_to_var, tail_values},
        Eval, EvalContext, EvalError, LuaValue,
    },
    syn::{Assignment, Expression, Var},
};

impl Eval for Assignment {
    type Return = ();

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        let Assignment { names, values } = self;
        assignment_values(context, values)
            .and_then(|values| multiple_assignment(context, names, values))
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
        .map(tail_values)
        .map(|values| values.chain(std::iter::repeat_with(|| LuaValue::Nil)))
}

fn multiple_assignment<'a, Context: EvalContext + ?Sized>(
    context: &mut Context,
    names: impl IntoIterator<Item = &'a Var>,
    values: impl Iterator<Item = LuaValue>,
) -> Result<(), EvalError> {
    for (name, value) in names.into_iter().zip(values) {
        assign_to_var(context, name, value)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use itertools::Itertools;
    use quickcheck::TestResult;

    use crate::{
        error::LuaError,
        lang::{
            Eval, EvalContext, EvalContextExt, EvalError, GlobalContext, LuaFunction, LuaKey,
            LuaValue, NaNLessTable, ReturnValue, TableRef, TableValue, TypeError,
        },
        lex::{Ident, Token},
        syn::{lua_parser, string_parser, Module, ParseError},
        test_util::vec_of_idents,
        util::NonEmptyVec, run_lua_test,
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
        let ret_value = ReturnValue::MultiValue(ret);
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

    #[quickcheck]
    fn assigning_to_a_table_member(key: LuaKey, value: LuaValue) -> Result<TestResult, LuaError> {
        if let LuaKey::Number(num) = key {
            if num.as_f64().is_nan() {
                return Ok(TestResult::discard());
            }
        }
        let module = string_parser::module("table[key] = value")?;

        let table = TableRef::from(TableValue::new());
        let mut context = GlobalContext::new();
        context.set("table", LuaValue::Table(table.clone()));
        context.set("value", value.clone());
        context.set("key", LuaValue::from(key.clone()));

        module.eval(&mut context)?;
        assert!(table.get(&key).total_eq(&value));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn assigning_to_existing_member_overrides_it(
        key: LuaKey,
        prev_value: LuaValue,
        value: LuaValue,
    ) -> Result<TestResult, LuaError> {
        let module = string_parser::module("table[key] = value")?;
        if let LuaKey::Number(num) = key {
            if num.as_f64().is_nan() {
                return Ok(TestResult::discard());
            }
        }

        let mut table = TableValue::new();
        table.set(key.clone(), prev_value);
        let table = TableRef::from(table);
        let mut context = GlobalContext::new();
        context.set("table", LuaValue::Table(table.clone()));
        context.set("value", value.clone());
        context.set("key", LuaValue::from(key.clone()));

        module.eval(&mut context)?;
        assert!(table.get(&key).total_eq(&value));
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn assigning_to_a_non_indexable_value_is_an_error(
        key: LuaKey,
        value: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if value.is_table() {
            return Ok(TestResult::discard());
        }

        let module = string_parser::module("value[key] = 42")?;
        let mut context = GlobalContext::new();
        context.set("key", key.into());
        context.set("value", value);
        let res = module.eval(&mut context);
        assert!(matches!(
            res,
            Err(EvalError::TypeError(TypeError::IsNotIndexable(_)))
        ));
        Ok(TestResult::passed())
    }

    #[test]
    fn assigning_to_a_nil_member_is_an_error() -> Result<(), LuaError> {
        let module = string_parser::module("tbl = {} tbl[nil] = 42")?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context);
        assert!(matches!(
            res,
            Err(EvalError::TypeError(TypeError::NilLookup))
        ));
        Ok(())
    }

    #[quickcheck]
    fn assigning_to_a_property_is_the_same_as_to_a_member_keyed_by_the_string_of_property_name(
        NaNLessTable(table): NaNLessTable,
        prop: Ident,
        value: LuaValue,
    ) -> Result<(), LuaError> {
        println!("{:?}\t{:?}\t{:?}", table, prop, value);
        let module = string_parser::module(&format!(
            "tbl1[\"{}\"] = value
            tbl2.{} = value",
            prop, prop
        ))?;
        let tbl1 = TableRef::from(table.clone());
        let tbl2 = TableRef::from(table);
        let mut context = GlobalContext::new();
        context.set("tbl1", LuaValue::Table(tbl1.clone()));
        context.set("tbl2", LuaValue::Table(tbl2.clone()));
        context.set("value", value.clone());

        module.eval(&mut context)?;
        drop(context);
        let tbl1 = tbl1.try_into_inner().unwrap();
        let tbl2 = tbl2.try_into_inner().unwrap();
        assert!(tbl1.total_eq(&tbl2));

        Ok(())
    }

    #[quickcheck]
    fn accessing_property_of_a_non_indexable_value_is_an_error(
        prop: Ident,
        value: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if value.is_table() {
            return Ok(TestResult::discard());
        }

        let module = string_parser::module(&format!("value.{} = 42", prop))?;
        let mut context = GlobalContext::new();
        context.set("value", value);
        let res = module.eval(&mut context);
        assert!(matches!(
            res,
            Err(EvalError::TypeError(TypeError::CannotAssignProperty { .. }))
        ));
        Ok(TestResult::passed())
    }

    #[test]
    fn lua_tests() -> Result<(), LuaError> {
        run_lua_test!("./assignment.test.lua", GlobalContext::new())
    }
}
