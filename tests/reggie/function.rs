use std::{cell::RefCell, rc::Rc};

use itertools::Itertools;
use luar_error::assert_type_error;
use luar_lex::Ident;
use luar_syn::lua_parser;
use non_empty::NonEmptyVec;
use quickcheck::TestResult;
use reggie::{eval_module, LuaError, LuaValue, Machine, NativeFunction, Strict, TypeError, call_block};

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

#[quickcheck]
fn fn_declaration_puts_function_in_scope(ident: Ident) -> Result<(), LuaError> {
    let module = lua_parser::module(&format!("function {}() end", ident))?;
    let mut machine = Machine::new();
    eval_module::<Strict<()>>(&module, &mut machine)?;
    assert!(machine.global_values.get(&ident).is_function());
    Ok(())
}

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

#[quickcheck]
fn function_multiple_returns(values: NonEmptyVec<LuaValue>) -> Result<TestResult, LuaError> {
    if values.len().get() > 16 {
        return Ok(TestResult::discard());
    }
    let idents: Vec<_> = (0..values.len().get())
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
    assert!(res.len() == values.len().get());
    assert!(res
        .into_iter()
        .zip(&values)
        .all(|(lhs, rhs)| lhs.total_eq(rhs)));
    Ok(TestResult::passed())
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
    let Strict(res) =
        eval_module::<Strict<(&LuaValue, &LuaValue, &LuaValue, &LuaValue)>>(&module, &mut machine)?;
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

#[test]
fn multiple_return_is_propagated() -> Result<(), LuaError> {
    let module = lua_parser::module(
        "function mult()
                return 1, 2
            end
            function m1()
                return mult()
            end
            function m2()
                return 3, mult()
            end
            function m3()
                return mult(), 3
            end",
    )?;
    let mut machine = Machine::new();
    eval_module::<Strict<()>>(&module, &mut machine)?;
    let expectations: [(&str, &[i32]); 4] = [
        ("mult", &[1, 2]),
        ("m1", &[1, 2]),
        ("m2", &[3, 1, 2]),
        ("m3", &[1, 3]),
    ];
    for (func, expected) in expectations {
        let block_id = machine.global_values.get(func).unwrap_lua_function();
        let res = call_block::<&[LuaValue]>(block_id, &mut machine)?;
        assert!(res
            .into_iter()
            .map(LuaValue::unwrap_int)
            .eq(expected.into_iter().cloned()));
    }

    Ok(())
}
