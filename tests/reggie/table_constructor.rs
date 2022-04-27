use luar_lex::{vec_of_idents, Ident};
use luar_syn::{lua_parser, Expression, Module, Return, TableConstructor, Var};
use non_empty::NonEmptyVec;
use reggie::{eval_module, LuaError, LuaValue, Machine, Strict, LuaKey, TableValue};

#[test]
fn empty_table_constructor_creates_empty_table() -> Result<(), LuaError> {
    let module = lua_parser::module("return {}")?;
    let mut machine = Machine::new();
    let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut machine)?;
    assert!(res.is_table());
    assert!(res.unwrap_table().is_empty());
    Ok(())
}

#[quickcheck]
fn list_table_constructor_creates_table_with_integer_indices_from_one_upwards(
    values: NonEmptyVec<LuaValue>,
) -> Result<(), LuaError> {
    let idents = vec_of_idents(values.len().get(), "value");

    let tbl = TableConstructor {
        lfield: idents
            .iter()
            .cloned()
            .map(Var::Named)
            .map(Expression::Variable)
            .collect(),
        ffield: vec![],
    };

    let module = Module {
        chunks: vec![],
        ret: Some(Return(vec![Expression::TableConstructor(tbl)])),
    };

    let mut machine = Machine::new();
    for (value, ident) in values.iter().cloned().zip(idents) {
        machine.global_values.set(ident, value);
    }

    let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut machine)?;

    let mut expected = TableValue::new();
    for (value, idx) in values.into_iter().zip(1..) {
        expected.set(LuaKey::Int(idx), value)
    }
    drop(machine);

    assert!(res.unwrap_table().unwrap_or_clone().total_eq(&expected));

    Ok(())
}

#[quickcheck]
fn key_value_pairs_constructor_creates_table_with_corresponding_key_value_association(
    values: NonEmptyVec<(Ident, LuaValue)>,
) -> Result<(), LuaError> {
    let idents = vec_of_idents(
        values.len().get(),
        "please_do_not_collide_with_autogenerated_idents",
    );

    let tbl = TableConstructor {
        lfield: vec![],
        ffield: values
            .iter()
            .map(|(ident, _)| ident)
            .cloned()
            .zip(
                idents
                    .iter()
                    .cloned()
                    .map(Var::Named)
                    .map(Expression::Variable),
            )
            .collect(),
    };

    let module = Module {
        chunks: vec![],
        ret: Some(Return(vec![Expression::TableConstructor(tbl)])),
    };

    let mut machine = Machine::new();
    for (value, ident) in values.iter().map(|(_, value)| value).cloned().zip(idents) {
        machine.global_values.set(ident, value);
    }

    let Strict(res) = eval_module::<Strict<LuaValue>>(&module, &mut machine)?;

    let mut expected = TableValue::new();
    for (key, value) in values {
        expected.set(LuaKey::String(key.into()), value)
    }
    drop(machine);

    assert!(res.unwrap_table().unwrap_or_clone().total_eq(&expected));

    Ok(())
}