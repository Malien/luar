use luar_syn::lua_parser;
use reggie::{eval_module, LuaError, LuaValue, Machine};

type LuaValue4 = (LuaValue, LuaValue, LuaValue, LuaValue);

#[quickcheck]
#[allow(non_snake_case)]
fn comparing_numbers_behave_according_to_IEEE754(lhs: f64, rhs: f64) -> Result<(), LuaError> {
    let module = lua_parser::module("return a > b, a < b, a >= b, a <= b")?;
    let mut machine = Machine::new();
    machine.global_values.set("a", LuaValue::Float(lhs));
    machine.global_values.set("b", LuaValue::Float(rhs));
    let expected = (
        LuaValue::from_bool(lhs > rhs),
        LuaValue::from_bool(lhs < rhs),
        LuaValue::from_bool(lhs >= rhs),
        LuaValue::from_bool(lhs <= rhs),
    );
    let res: LuaValue4 = eval_module(&module, &mut machine)?;
    assert_eq!(res, expected);
    Ok(())
}

#[quickcheck]
fn comparing_strings_orders_then_lexicographically(
    lhs: String,
    rhs: String,
) -> Result<(), LuaError> {
    let module = lua_parser::module("return a > b, a < b, a >= b, a <= b")?;
    let mut machine = Machine::new();
    machine
        .global_values
        .set("a", LuaValue::string(lhs.clone()));
    machine
        .global_values
        .set("b", LuaValue::string(rhs.clone()));
    let expected = (
        LuaValue::from_bool(lhs > rhs),
        LuaValue::from_bool(lhs < rhs),
        LuaValue::from_bool(lhs >= rhs),
        LuaValue::from_bool(lhs <= rhs),
    );
    let res: LuaValue4 = eval_module(&module, &mut machine)?;
    assert_eq!(res, expected);
    Ok(())
}

#[quickcheck]
fn comparing_strings_and_numbers_coerces_numbers_to_strings(
    str: String,
    num: f64,
) -> Result<(), LuaError> {
    let module = lua_parser::module("return a > b, a < b, a >= b, a <= b")?;
    let mut machine = Machine::new();

    {
        machine
            .global_values
            .set("a", LuaValue::string(&str));
        machine.global_values.set("b", LuaValue::Float(num));
        let lhs = &str;
        let rhs = &format!("{}", num);
        let expected = (
            LuaValue::from_bool(lhs > rhs),
            LuaValue::from_bool(lhs < rhs),
            LuaValue::from_bool(lhs >= rhs),
            LuaValue::from_bool(lhs <= rhs),
        );
        let res: LuaValue4 = eval_module(&module, &mut machine)?;
        assert_eq!(res, expected);
    }
    {
        machine.global_values.set("a", LuaValue::Float(num));
        machine
            .global_values
            .set("b", LuaValue::string(&str));
        let lhs = &format!("{}", num);
        let rhs = &str;
        let expected = (
            LuaValue::from_bool(lhs > rhs),
            LuaValue::from_bool(lhs < rhs),
            LuaValue::from_bool(lhs >= rhs),
            LuaValue::from_bool(lhs <= rhs),
        );
        let res: LuaValue4 = eval_module(&module, &mut machine)?;
        assert_eq!(res, expected);
    }

    Ok(())
}

#[quickcheck]
fn values_other_than_numbers_and_strings_are_not_comparable(val: LuaValue) -> Result<(), LuaError> {
    let ops = [">", "<", ">=", "<="];
    let modules = IntoIterator::into_iter(ops)
        .flat_map(|op| {
            [
                format!("return 1 {} value", op),
                format!("return value {} 1", op),
            ]
        })
        .map(|str| lua_parser::module(&str))
        .collect::<Result<Vec<_>, _>>()?;
    let mut machine = Machine::new();
    let is_comparable = val.is_comparable();
    machine.global_values.set("value", val);

    for module in modules {
        let res = eval_module::<()>(&module, &mut machine);
        if is_comparable {
            assert!(res.is_ok());
        } else {
            assert!(res.is_err());
        }
    }

    Ok(())
}
