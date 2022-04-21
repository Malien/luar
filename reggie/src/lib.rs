#[cfg(test)]
#[cfg(feature = "quickcheck")]
#[macro_use]
extern crate quickcheck_macros;

pub(crate) mod compiler;
pub(crate) mod eq_with_nan;
pub(crate) mod ids;
pub(crate) mod machine;
pub(crate) mod meta;
pub(crate) mod ops;
pub(crate) mod runtime;
pub mod stdlib;
pub mod value;

pub use machine::Machine;
pub use value::*;

pub type LuaError = luar_error::LuaError<LuaValue>;
pub type EvalError = luar_error::EvalError<LuaValue>;
pub type TypeError = luar_error::TypeError<LuaValue>;
pub type ArithmeticError = luar_error::ArithmeticError<LuaValue>;

use value::FromReturn;

pub fn eval_str<'a, T: FromReturn<'a>>(
    module_str: &str,
    machine: &'a mut Machine,
) -> Result<T, LuaError> {
    let module = luar_syn::lua_parser::module(module_str)?;
    eval_module(&module, machine).map_err(LuaError::from)
}

pub fn eval_module<'a, T: FromReturn<'a>>(
    module: &luar_syn::Module,
    machine: &'a mut Machine,
) -> Result<T, EvalError> {
    let compiled_module = compiler::compile_module(&module, &mut machine.global_values);
    runtime::call_module(compiled_module, machine)
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        eval_module, eval_str, value::Strict, LuaError, LuaValue, Machine, NativeFunction,
        TypeError,
    };
    use luar_error::assert_type_error;
    use luar_syn::lua_parser;

    #[cfg(feature = "quickcheck")]
    use itertools::Itertools;
    #[cfg(feature = "quickcheck")]
    use luar_lex::{Ident, NumberLiteral, StringLiteral, Token};
    #[cfg(feature = "quickcheck")]
    use non_empty::NonEmptyVec;
    #[cfg(feature = "quickcheck")]
    use quickcheck::TestResult;

    #[test]
    fn eval_empty() -> Result<(), LuaError> {
        let mut machine = Machine::new();
        eval_str("", &mut machine)
    }

    #[test]
    fn eval_nil() -> Result<(), LuaError> {
        let mut machine = Machine::new();
        assert_eq!(
            eval_str::<Strict<LuaValue>>("return nil", &mut machine)?.0,
            LuaValue::Nil
        );
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn eval_number_literal(num: f64) -> Result<TestResult, LuaError> {
        use crate::eq_with_nan::eq_with_nan;
        use luar_syn::unspanned_lua_token_parser;

        if !num.is_finite() {
            return Ok(TestResult::discard());
        }
        let module =
            unspanned_lua_token_parser::module([Token::Return, Token::Number(NumberLiteral(num))])?;
        let mut machine = Machine::new();
        let res = eval_module::<Strict<LuaValue>>(&module, &mut machine)?
            .0
            .number_as_f64()
            .unwrap();
        assert!(eq_with_nan(res, num));
        Ok(TestResult::passed())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn eval_string_literal(str: String) -> Result<(), LuaError> {
        use luar_syn::unspanned_lua_token_parser;

        let module = unspanned_lua_token_parser::module([
            Token::Return,
            Token::String(StringLiteral(str.clone())),
        ])?;
        let mut context = Machine::new();
        assert_eq!(
            eval_module::<Strict<&LuaValue>>(&module, &mut context)?.0,
            &LuaValue::String(str)
        );
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn value_is_equal_to_itself(value: LuaValue) -> Result<TestResult, LuaError> {
        if let LuaValue::Float(num) = value {
            if num.is_nan() {
                // NaN does not equal itself
                return Ok(TestResult::discard());
            }
        }

        let mut machine = Machine::new();
        machine.global_values.set("value", value);
        let res = eval_str("return value == value", &mut machine)?;
        assert_eq!(LuaValue::true_value(), res);
        Ok(TestResult::passed())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn different_values_do_not_equal_themselves(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<(), LuaError> {
        let expected = LuaValue::from_bool(lhs == rhs);
        let mut machine = Machine::new();
        machine.global_values.set("lhs", lhs);
        machine.global_values.set("rhs", rhs);
        let res = eval_str("return lhs == rhs", &mut machine)?;
        assert_eq!(expected, res);
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn not_equals_is_the_negation_of_equality(
        lhs: LuaValue,
        rhs: LuaValue,
    ) -> Result<(), LuaError> {
        let mut machine = Machine::new();
        machine.global_values.set("lhs", lhs);
        machine.global_values.set("rhs", rhs);
        let res = eval_str("return (not (lhs ~= rhs)) == (lhs == rhs)", &mut machine)?;
        assert_eq!(LuaValue::true_value(), res);
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
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

    #[cfg(feature = "quickcheck")]
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

    // #[cfg(feature = "quickcheck")]
    // fn multi_return_fn(ret: NonEmptyVec<LuaValue>) -> LuaValue {
    //     let ret_value = ReturnValue::MultiValue(ret);
    //     let lua_fn = LuaFunction::new(move |_, _| Ok(ret_value.clone()));
    //     LuaValue::Function(lua_fn)
    // }

    #[cfg(feature = "quickcheck")]
    fn assign_values(
        global: &mut crate::machine::GlobalValues,
        names: impl IntoIterator<Item = Ident>,
        values: impl IntoIterator<Item = LuaValue>,
    ) {
        for (name, value) in names.into_iter().zip(values) {
            global.set(name, value);
        }
    }

    #[cfg(feature = "quickcheck")]
    fn assert_multiple_assignment(
        global: &crate::machine::GlobalValues,
        idents: Vec<Ident>,
        values: Vec<LuaValue>,
    ) {
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

    #[cfg(feature = "quickcheck")]
    fn put_dummy_values<'a>(
        values: &mut crate::machine::GlobalValues,
        idents: impl IntoIterator<Item = &'a Ident>,
    ) {
        for ident in idents {
            values.set(ident.clone(), LuaValue::Int(42));
        }
    }

    #[cfg(feature = "quickcheck")]
    fn vec_of_idents(len: usize, prefix: &str) -> Vec<luar_lex::Ident> {
        (0..len)
            .into_iter()
            .map(|i| format!("{}{}", prefix, i))
            .map(luar_lex::Ident::new)
            .collect()
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    #[allow(unstable_name_collisions)]
    fn eval_multiple_assignment(
        idents: std::collections::HashSet<Ident>,
        values: non_empty::NonEmptyVec<LuaValue>,
    ) -> Result<TestResult, LuaError> {
        if idents.len() == 0 {
            return Ok(TestResult::discard());
        }
        // Make iteration order deterministic

        use luar_syn::unspanned_lua_token_parser;

        let idents: Vec<_> = idents.into_iter().collect();
        let value_idents = vec_of_idents(values.len(), "value");

        let tokens: Vec<_> =
            multiple_assignment_tokens(idents.iter().cloned(), value_idents.iter().cloned())
                .collect();
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

    // #[cfg(feature = "quickcheck")]
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

    // #[cfg(feature = "quickcheck")]
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

    // #[cfg(feature = "quickcheck")]
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

    // #[cfg(feature = "quickcheck")]
    // #[quickcheck]
    // fn assigning_to_a_table_member(key: LuaKey, value: LuaValue) -> Result<TestResult, LuaError> {
    //     if let LuaKey::Number(num) = key {
    //         if num.as_f64().is_nan() {
    //             return Ok(TestResult::discard());
    //         }
    //     }
    //     let module = lua_parser::module("table[key] = value")?;

    //     let table = TableRef::from(TableValue::new());
    //     let mut context = GlobalContext::new();
    //     context.set("table", LuaValue::Table(table.clone()));
    //     context.set("value", value.clone());
    //     context.set("key", LuaValue::from(key.clone()));

    //     ast_vm::eval_module(&module, &mut context)?;
    //     assert!(table.get(&key).total_eq(&value));
    //     Ok(TestResult::passed())
    // }

    // #[cfg(feature = "quickcheck")]
    // #[quickcheck]
    // fn assigning_to_existing_member_overrides_it(
    //     key: LuaKey,
    //     prev_value: LuaValue,
    //     value: LuaValue,
    // ) -> Result<TestResult, LuaError> {
    //     let module = lua_parser::module("table[key] = value")?;
    //     if let LuaKey::Number(num) = key {
    //         if num.as_f64().is_nan() {
    //             return Ok(TestResult::discard());
    //         }
    //     }

    //     let mut table = TableValue::new();
    //     table.set(key.clone(), prev_value);
    //     let table = TableRef::from(table);
    //     let mut context = GlobalContext::new();
    //     context.set("table", LuaValue::Table(table.clone()));
    //     context.set("value", value.clone());
    //     context.set("key", LuaValue::from(key.clone()));

    //     ast_vm::eval_module(&module, &mut context)?;
    //     assert!(table.get(&key).total_eq(&value));
    //     Ok(TestResult::passed())
    // }

    // #[cfg(feature = "quickcheck")]
    // #[quickcheck]
    // fn assigning_to_a_non_indexable_value_is_an_error(
    //     key: LuaKey,
    //     value: LuaValue,
    // ) -> Result<TestResult, LuaError> {
    //     if value.is_table() {
    //         return Ok(TestResult::discard());
    //     }

    //     let module = lua_parser::module("value[key] = 42")?;
    //     let mut context = GlobalContext::new();
    //     context.set("key", key.into());
    //     context.set("value", value);
    //     let res = ast_vm::eval_module(&module, &mut context);
    //     assert_type_error!(TypeError::IsNotIndexable(_), res);
    //     Ok(TestResult::passed())
    // }

    #[test]
    fn assigning_to_a_nil_member_is_an_error() -> Result<(), LuaError> {
        let module = lua_parser::module("tbl = {} tbl[nil] = 42")?;
        let mut machine = Machine::new();
        let res = eval_module::<Strict<()>>(&module, &mut machine);
        assert_type_error!(TypeError::NilLookup, res);
        Ok(())
    }

    // #[cfg(feature = "quickcheck")]
    // #[quickcheck]
    // fn assigning_to_a_property_is_the_same_as_to_a_member_keyed_by_the_string_of_property_name(
    //     NaNLessTable(table): NaNLessTable,
    //     prop: Ident,
    //     value: LuaValue,
    // ) -> Result<(), LuaError> {
    //     let module = lua_parser::module(&format!(
    //         "tbl1[\"{}\"] = value
    //         tbl2.{} = value",
    //         prop, prop
    //     ))?;
    //     let tbl1 = TableRef::from(table.clone());
    //     let tbl2 = TableRef::from(table);
    //     let mut context = GlobalContext::new();
    //     context.set("tbl1", LuaValue::Table(tbl1.clone()));
    //     context.set("tbl2", LuaValue::Table(tbl2.clone()));
    //     context.set("value", value.clone());

    //     ast_vm::eval_module(&module, &mut context)?;
    //     drop(context);
    //     let tbl1 = tbl1.try_into_inner().unwrap();
    //     let tbl2 = tbl2.try_into_inner().unwrap();
    //     assert!(tbl1.total_eq(&tbl2));

    //     Ok(())
    // }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn accessing_property_of_a_non_indexable_value_is_an_error(
        prop: Ident,
        value: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if value.is_table() {
            return Ok(TestResult::discard());
        }

        let module = lua_parser::module(&format!("value.{} = 42", prop))?;
        let mut machine = Machine::new();
        machine.global_values.set("value", value);
        let res = eval_module::<Strict<()>>(&module, &mut machine);
        assert_type_error!(TypeError::CannotAssignProperty { .. }, res);
        Ok(TestResult::passed())
    }

    #[test]
    fn eval_fn_call() -> Result<(), LuaError> {
        let module = lua_parser::module("myfn(42)")?;
        let called_with = Rc::new(RefCell::new(0));
        let myfn = NativeFunction::new({
            let called_with = Rc::clone(&called_with);
            move |first_arg: LuaValue| {
                let mut called = called_with.borrow_mut();
                *called = first_arg.unwrap_int();
            }
        });
        let mut machine = Machine::new();
        machine
            .global_values
            .set("myfn", LuaValue::NativeFunction(myfn));
        eval_module::<Strict<()>>(&module, &mut machine)?;
        let called = called_with.borrow();
        assert_eq!(*called, 42);
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn eval_fn_return(ret_value: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module("return myfn()")?;
        let mut machine = Machine::new();
        let myfn = NativeFunction::new({
            let ret_value = ret_value.clone();
            move || ret_value.clone()
        });
        machine
            .global_values
            .set("myfn", LuaValue::NativeFunction(myfn));
        let Strict(res) = eval_module(&module, &mut machine)?;
        assert!(ret_value.total_eq(&res));
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn calling_not_a_function_value_is_an_error(value: LuaValue) -> Result<TestResult, LuaError> {
        if value.is_function() {
            return Ok(TestResult::discard());
        }

        let module = lua_parser::module("value()")?;
        let mut machine = Machine::new();
        machine.global_values.set("value", value);
        let res = eval_module::<Strict<()>>(&module, &mut machine);
        assert_type_error!(TypeError::IsNotCallable(_), res);
        Ok(TestResult::passed())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn eval_fn_call_multiple_returns(value1: LuaValue, value2: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module("return myfn()")?;
        let mut machine = Machine::new();
        let ret_values = (value1.clone(), value2.clone());
        let myfn = NativeFunction::new({
            let ret_values = ret_values.clone();
            move || ret_values.clone()
        });
        machine
            .global_values
            .set("myfn", LuaValue::NativeFunction(myfn));
        let Strict((res1, res2)) =
            eval_module::<Strict<(&LuaValue, &LuaValue)>>(&module, &mut machine)?;
        assert!(res1.total_eq(&value1));
        assert!(res2.total_eq(&value2));
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn fn_declaration_puts_function_in_scope(ident: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("function {}() end", ident))?;
        let mut machine = Machine::new();
        eval_module::<Strict<()>>(&module, &mut machine)?;
        assert!(machine.global_values.get(&ident).is_function());
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn fn_declaration_return(ret_value: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function myfn() return value end
            return myfn()",
        )?;
        let mut machine = Machine::new();
        machine.global_values.set("value", ret_value.clone());
        let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut machine)?;
        assert!(machine.global_values.get("myfn").is_function());
        assert!(res.total_eq(&ret_value));

        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn function_multiple_returns(values: NonEmptyVec<LuaValue>) -> Result<(), LuaError> {
        let idents: Vec<_> = (0..values.len())
            .into_iter()
            .map(|i| format!("value{}", i))
            .map(Ident::new)
            .collect();
        let idents_str = idents.iter().join(", ");
        let module = lua_parser::module(&format!(
            "function myfn()
                return {}
            end
            return myfn()",
            idents_str
        ))?;
        let mut machine = Machine::new();
        for (value, ident) in values.iter().zip(idents) {
            machine.global_values.set(ident, value.clone());
        }
        let res = eval_module::<&[LuaValue]>(&module, &mut machine)?;
        assert!(res.len() == values.len());
        assert!(res
            .into_iter()
            .zip(&values)
            .all(|(lhs, rhs)| lhs.total_eq(rhs)));
        Ok(())
    }

    #[test]
    fn function_executes_side_effect() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "executed = nil
            function myfn() 
                executed = 1
            end
            myfn()
            return executed",
        )?;
        let mut machine = Machine::new();
        let Strict(res) = eval_module::<Strict<bool>>(&module, &mut machine)?;
        assert!(res);
        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn local_declarations_stay_local(ident: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!(
            "{} = \"global\"
            function myfn()
                local {} = \"local\"
                return {}
            end
            return myfn(), {}",
            ident, ident, ident, ident
        ))?;
        let mut machine = Machine::new();
        let res = eval_module::<Result<(&str, &str), _>>(&module, &mut machine)?.unwrap();
        let expected = ("local", "global");
        assert_eq!(res, expected);

        Ok(())
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn arguments_passed_in_are_defined_as_local_variables_inside_fn(
        value: LuaValue,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function myfn(arg)
                return arg
            end
            return myfn(value), arg",
        )?;
        let mut machine = Machine::new();
        machine.global_values.set("value", value.clone());
        let Strict((func_return, arg)) =
            eval_module::<Strict<(&LuaValue, &LuaValue)>>(&module, &mut machine)?;
        assert!(func_return.total_eq(&value));
        assert_eq!(arg, &LuaValue::Nil);
        Ok(())
    }

    #[test]
    fn not_passed_arguments_are_set_to_nil() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function myfn(a, b, c, d)
                return a, b, c, d
            end
            return myfn(1, 2)",
        )?;
        let mut machine = Machine::new();
        let Strict(res) = eval_module::<Strict<(&LuaValue, &LuaValue, &LuaValue, &LuaValue)>>(
            &module,
            &mut machine,
        )?;
        let expected = (
            &LuaValue::Int(1),
            &LuaValue::Int(2),
            &LuaValue::Nil,
            &LuaValue::Nil,
        );
        assert_eq!(res, expected);
        Ok(())
    }

    #[test]
    fn passing_more_arguments_than_stated_just_gets_arglist_truncated() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "function myfn(a, b)
                return a, b
            end
            return myfn(1, 2, 3, 4)",
        )?;
        let mut machine = Machine::new();
        let Strict(res) = eval_module::<Strict<(&LuaValue, &LuaValue)>>(&module, &mut machine)?;
        let expected = (&LuaValue::Int(1), &LuaValue::Int(2));
        assert_eq!(res, expected);
        Ok(())
    }

    // #[test]
    // fn multiple_return_is_propagated() -> Result<(), LuaError> {
    //     let module = lua_parser::module(
    //         "function mult()
    //             return 1, 2
    //         end
    //         function m1()
    //             return mult()
    //         end
    //         function m2()
    //             return 3, mult()
    //         end
    //         function m3()
    //             return mult(), 3
    //         end",
    //     )?;
    //     let mut context = GlobalContext::new();
    //     ast_vm::eval_module(&module, &mut context)?;
    //     let expectations = [
    //         ("mult", &[1, 2]),
    //         ("m1", &[1, 2]),
    //         ("m2", &[3, 1, 2]),
    //         ("m3", &[1, 3]),
    //     ];
    //     for (func, expected) in expectations {
    //         let res = context
    //             .get(func)
    //             .unwrap_function_ref()
    //             .clone()
    //             .call(&mut context, &[])?;
    //         assert_eq!(res, expected);
    //     }

    //     Ok(())
    // }
}
