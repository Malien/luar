use luar_lex::Ident;
use luar_syn::lua_parser;
use quickcheck_macros::quickcheck;
use reggie::{LuaValue, LuaError, Machine, eval_module, Strict};

#[quickcheck]
fn local_decl_does_not_behave_like_global_assignment_in_global_scope(
    ident: Ident,
    value: LuaValue,
) -> Result<(), LuaError> {
    let module = lua_parser::module(&format!("local {} = value", ident))?;
    let mut machine = Machine::new();
    machine.global_values.set("value", value.clone());
    eval_module::<Strict<()>>(&module, &mut machine)?;
    assert_eq!(machine.global_values.get(&ident), &LuaValue::Nil);
    Ok(())
}

#[quickcheck]
fn redeclaring_local_does_nothing(ident: Ident, value: LuaValue) -> Result<(), LuaError> {
    let module = lua_parser::module(&format!(
        "local {} = value
            local {}
            return {}",
        ident, ident, ident
    ))?;
    let mut machine = Machine::new();
    machine.global_values.set("value", value.clone());
    let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut machine)?;
    assert!(res.total_eq(&value));
    Ok(())
}

#[quickcheck]
fn redeclaring_local_with_new_value_does_nothing(
    ident: Ident,
    value1: LuaValue,
    value2: LuaValue,
) -> Result<(), LuaError> {
    let module = lua_parser::module(&format!(
        "local {} = value1
            local {} = value2
            return {}",
        ident, ident, ident
    ))?;
    let mut machine = Machine::new();
    machine.global_values.set("value1", value1.clone());
    machine.global_values.set("value2", value2);
    let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut machine)?;
    assert!(res.total_eq(&value1));
    Ok(())
}

#[quickcheck]
fn set_global_values_behave_like_local_declarations(
    ident: Ident,
    value1: LuaValue,
    value2: LuaValue,
) -> Result<(), LuaError> {
    let module = lua_parser::module(&format!(
        "{} = value1
            local {} = value2
            return {}",
        ident, ident, ident
    ))?;
    let mut machine = Machine::new();
    machine.global_values.set("value1", value1.clone());
    machine.global_values.set("value2", value2);
    eval_module::<()>(&module, &mut machine)?;
    assert!(machine.global_values.get(&ident).total_eq(&value1));
    Ok(())
}

#[quickcheck]
fn set_global_value_cannot_be_undeclared(
    ident: Ident,
    value1: LuaValue,
    value2: LuaValue,
) -> Result<(), LuaError> {
    let module = lua_parser::module(&format!(
        "{} = value1
            {} = nil
            local {} = value2
            return {}",
        ident, ident, ident, ident
    ))?;
    let mut machine = Machine::new();
    machine.global_values.set("value1", value1);
    machine.global_values.set("value2", value2);
    eval_module::<()>(&module, &mut machine)?;
    assert_eq!(machine.global_values.get(&ident), &LuaValue::Nil);
    Ok(())
}

#[test]
fn local_var_in_global_context_is_not_accessible_from_other_function_contexts(
) -> Result<(), LuaError> {
    let module = lua_parser::module(
        "local foo = 42
            function bar() return foo end
            return bar()",
    )?;
    let mut context = Machine::new();
    let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut context)?;
    assert_eq!(res, LuaValue::Nil);
    Ok(())
}

#[test]
fn global_var_cannot_be_redeclared_local() -> Result<(), LuaError> {
    let module = lua_parser::module(
        "foo = 69
            local foo = 42
            function bar() return foo end
            return foo, bar()",
    )?;
    let mut machine = Machine::new();
    let Strict((foo, bar_res)) =
        eval_module::<Strict<(LuaValue, LuaValue)>>(&module, &mut machine)?;
    assert_eq!(foo, LuaValue::Int(69));
    assert_eq!(bar_res, LuaValue::Int(69));
    Ok(())
}

#[test]
fn local_vars_do_not_leak_through_function_calls() -> Result<(), LuaError> {
    let module = lua_parser::module(
        "
            function foo()
                return a
            end
            
            function bar()
                local a = 42
                return foo()
            end

            return bar()",
    )?;
    let mut context = Machine::new();
    let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut context)?;
    assert_eq!(res, LuaValue::Nil);
    Ok(())
}

#[test]
fn local_scopes_are_different() -> Result<(), LuaError> {
    let module = lua_parser::module(
        "
            if 1 then
                local foo = 42
            end

            if 1 then
                return foo
            end
            return 69
        ",
    )?;
    let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut Machine::new())?;
    assert_eq!(res, LuaValue::Nil);
    Ok(())
}

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