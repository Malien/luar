use itertools::Itertools;
use luar_lex::{Ident, Token};
use luar_syn::lua_parser;
use quickcheck::TestResult;
use reggie::{eval_module, GlobalValues, LuaError, LuaValue, Machine, Strict};

#[quickcheck]
fn eval_single_assignment(ident: Ident, v1: LuaValue, v2: LuaValue) -> Result<(), LuaError> {
    let module = lua_parser::module(&format!("{} = value", ident))?;
    let mut machine = Machine::new();
    assert_eq!(machine.global_values.get(&ident), &LuaValue::Nil);
    machine.global_values.set("value", v1.clone());
    eval_module::<()>(&module, &mut machine)?;
    assert!(machine.global_values.get(&ident).total_eq(&v1));
    machine.global_values.set("value", v2.clone());
    eval_module::<()>(&module, &mut machine)?;
    assert!(machine.global_values.get(&ident).total_eq(&v2));
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

// fn multi_return_fn(ret: NonEmptyVec<LuaValue>) -> LuaValue {
//     let ret_value = ReturnValue::MultiValue(ret);
//     let lua_fn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
//     LuaValue::Function(lua_fn)
// }

fn assign_values(
    global: &mut GlobalValues,
    names: impl IntoIterator<Item = Ident>,
    values: impl IntoIterator<Item = LuaValue>,
) {
    for (name, value) in names.into_iter().zip(values) {
        global.set(name, value);
    }
}

fn assert_multiple_assignment(global: &GlobalValues, idents: Vec<Ident>, values: Vec<LuaValue>) {
    if idents.len() > values.len() {
        for ident in &idents[values.len()..] {
            assert_eq!(global.get(ident), &LuaValue::Nil);
        }
    }

    for (ident, value) in idents.into_iter().zip(values) {
        let res = global.get(&ident).total_eq(&value);
        assert!(res);
    }
}

fn put_dummy_values<'a>(values: &mut GlobalValues, idents: impl IntoIterator<Item = &'a Ident>) {
    for ident in idents {
        values.set(ident.clone(), LuaValue::Int(42));
    }
}

fn vec_of_idents(len: usize, prefix: &str) -> Vec<luar_lex::Ident> {
    (0..len)
        .into_iter()
        .map(|i| format!("{}{}", prefix, i))
        .map(luar_lex::Ident::new)
        .collect()
}

#[quickcheck]
#[allow(unstable_name_collisions)]
fn eval_multiple_assignment(
    idents: std::collections::HashSet<Ident>,
    values: non_empty::NonEmptyVec<LuaValue>,
) -> Result<TestResult, LuaError> {
    if idents.len() == 0 || idents.len() > 16 {
        return Ok(TestResult::discard());
    }
    // Make iteration order deterministic

    use luar_syn::unspanned_lua_token_parser;

    let idents: Vec<_> = idents.into_iter().collect();
    let value_idents = vec_of_idents(values.len().get(), "value");

    let tokens: Vec<_> =
        multiple_assignment_tokens(idents.iter().cloned(), value_idents.iter().cloned()).collect();
    let module = unspanned_lua_token_parser::module(tokens)?;

    let mut machine = Machine::new();
    assign_values(
        &mut machine.global_values,
        value_idents.iter().cloned(),
        values.iter().cloned(),
    );
    put_dummy_values(&mut machine.global_values, &idents);

    eval_module::<Strict<()>>(&module, &mut machine)?;
    assert_multiple_assignment(&machine.global_values, idents, values.into());

    Ok(TestResult::passed())
}

// #[quickcheck]
// #[allow(unstable_name_collisions)]
// fn multiple_expanded_assignment(
//     idents: HashSet<Ident>,
//     mut left_values: Vec<LuaValue>,
//     right_values: NonEmptyVec<LuaValue>,
// ) -> Result<TestResult, LuaError> {
//     if idents.len() == 0 {
//         return Ok(TestResult::discard());
//     }
//     // Make iteration order deterministic
//     let idents: Vec<_> = idents.into_iter().collect();
//     let value_idents = vec_of_idents(left_values.len(), "value");

//     let tokens: Vec<_> = idents
//         .iter()
//         .cloned()
//         .map(Token::Ident)
//         .intersperse_with(|| Token::Comma)
//         .chain(std::iter::once(Token::Assignment))
//         .chain(
//             value_idents
//                 .iter()
//                 .cloned()
//                 .map(Token::Ident)
//                 .interleave_shortest(std::iter::repeat_with(|| Token::Comma)),
//         )
//         .chain([
//             Token::Ident(Ident::new("myfn")),
//             Token::OpenRoundBracket,
//             Token::CloseRoundBracket,
//         ])
//         .collect();
//     let module = unspanned_lua_token_parser::module(tokens)?;

//     let mut context = GlobalContext::new();
//     context.set("myfn", multi_return_fn(right_values.clone()));
//     assign_values(&mut context, value_idents, left_values.iter().cloned());
//     put_dummy_values(&mut context, &idents);
//     ast_vm::eval_module(&module, &mut context)?;

//     left_values.extend(right_values.into_iter());
//     assert_multiple_assignment(&mut context, idents.into(), left_values);

//     Ok(TestResult::passed())
// }

// #[allow(unstable_name_collisions)]
// fn multiple_expanded_assignment_in_the_middle_module(
//     idents: impl Iterator<Item = Ident>,
//     left_idents: impl Iterator<Item = Ident>,
//     right_idents: impl Iterator<Item = Ident>,
//     fn_name: Ident,
// ) -> Result<luar_syn::Module, luar_syn::RawParseError> {
//     use luar_syn::unspanned_lua_token_parser;

//     let tokens: Vec<_> = idents
//         .map(Token::Ident)
//         .intersperse_with(|| Token::Comma)
//         .chain(std::iter::once(Token::Assignment))
//         .chain(
//             left_idents
//                 .map(Token::Ident)
//                 .interleave_shortest(std::iter::repeat_with(|| Token::Comma)),
//         )
//         .chain([
//             Token::Ident(fn_name),
//             Token::OpenRoundBracket,
//             Token::CloseRoundBracket,
//             Token::Comma,
//         ])
//         .chain(
//             right_idents
//                 .map(Token::Ident)
//                 .intersperse_with(|| Token::Comma),
//         )
//         .collect();
//     unspanned_lua_token_parser::module(tokens)
// }

// #[quickcheck]
// fn multiple_expanded_assignment_in_the_middle(
//     idents: HashSet<Ident>,
//     left_values: Vec<LuaValue>,
//     multi_value: NonEmptyVec<LuaValue>,
//     right_values: NonEmptyVec<LuaValue>,
// ) -> Result<TestResult, LuaError> {
//     if idents.len() == 0 {
//         return Ok(TestResult::discard());
//     }
//     // Make iteration order deterministic
//     let idents: Vec<_> = idents.into_iter().collect();
//     let left_idents = vec_of_idents(left_values.len(), "left_value");
//     let right_idents = vec_of_idents(right_values.len(), "right_value");

//     let module = multiple_expanded_assignment_in_the_middle_module(
//         idents.iter().cloned(),
//         left_idents.iter().cloned(),
//         right_idents.iter().cloned(),
//         Ident::new("myfn"),
//     )?;

//     let mut context = GlobalContext::new();
//     context.set("myfn", multi_return_fn(multi_value.clone()));
//     assign_values(&mut context, left_idents, left_values.iter().cloned());
//     assign_values(&mut context, right_idents, right_values.iter().cloned());
//     put_dummy_values(&mut context, &idents);

//     ast_vm::eval_module(&module, &mut context)?;

//     let resulting_values: Vec<LuaValue> = left_values
//         .into_iter()
//         .chain(std::iter::once(multi_value.move_first()))
//         .chain(right_values)
//         .collect();
//     assert_multiple_assignment(&context, idents, resulting_values);

//     Ok(TestResult::passed())
// }
