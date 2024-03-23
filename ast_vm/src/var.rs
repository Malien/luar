use crate::{
    lang::{LocalScope, LuaKey, LuaValue, ScopeHolder},
    EvalError, TypeError,
};
use luar_lex::Ident;
use luar_syn::Var;

use super::eval_expr;

pub(crate) fn eval_var(
    var: &Var,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<LuaValue, EvalError> {
    match var {
        Var::Named(ident) => Ok(scope.get(ident).clone()),
        Var::MemberLookup { from, value } => {
            let from = eval_var(from, scope)?;
            let key = eval_expr(value, scope)?.first_value();
            member_lookup(from, key)
        }
        Var::PropertyAccess { from, property } => {
            let from = eval_var(from, scope)?;
            property_access(from, property.clone())
        }
    }
    .map_err(EvalError::from)
}

fn member_lookup(value: LuaValue, key: LuaValue) -> Result<LuaValue, TypeError> {
    if let LuaValue::Table(table) = value {
        Ok(LuaKey::new(key)
            .map(|key| table.get(&key))
            .unwrap_or(LuaValue::Nil))
    } else {
        Err(TypeError::IsNotIndexable(value))
    }
}

fn property_access(value: LuaValue, property: Ident) -> Result<LuaValue, TypeError> {
    if let LuaValue::Table(table) = value {
        let key = LuaKey::string(property);
        Ok(table.get(&key))
    } else {
        Err(TypeError::CannotAccessProperty {
            property,
            of: value,
        })
    }
}

pub(crate) fn assign_to_var(
    scope: &mut LocalScope<impl ScopeHolder>,
    var: &Var,
    value: LuaValue,
) -> Result<(), EvalError> {
    match var {
        Var::Named(ident) => Ok(scope.set(ident.clone(), value)),
        Var::MemberLookup { from, value: key } => {
            let from = eval_var(from, scope)?;
            let key = eval_expr(key, scope)?.first_value();
            assign_to_value_member(from, key, value)
        }
        Var::PropertyAccess { from, property } => {
            let from = eval_var(from, scope)?;
            assign_to_value_property(from, property.clone(), value)
        }
    }
    .map_err(EvalError::from)
}

fn assign_to_value_member(of: LuaValue, key: LuaValue, value: LuaValue) -> Result<(), TypeError> {
    if let LuaValue::Table(mut table) = of {
        if let Some(key) = LuaKey::new(key) {
            table.set(key, value);
            Ok(())
        } else {
            Err(TypeError::NilAssign(value))
        }
    } else {
        Err(TypeError::IsNotIndexable(of))
    }
}

fn assign_to_value_property(
    of: LuaValue,
    property: Ident,
    value: LuaValue,
) -> Result<(), TypeError> {
    if let LuaValue::Table(mut table) = of {
        let key = LuaKey::string(property);
        table.set(key, value);
        Ok(())
    } else {
        Err(TypeError::CannotAssignProperty { property, of })
    }
}

#[cfg(test)]
mod test {
    use crate as ast_vm;
    use crate::{
        lang::{GlobalContext, LuaKey, LuaValue, ReturnValue, TableValue},
        LuaError, TypeError,
    };
    use luar_error::assert_type_error;
    use luar_lex::Ident;
    use luar_syn::lua_parser;
    use quickcheck::TestResult;

    #[quickcheck]
    fn eval_ident_on_global(value: LuaValue, ident: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("return {}", ident))?;
        let mut context = GlobalContext::new();
        assert_eq!(
            ast_vm::eval_module(&module, &mut context)?,
            ReturnValue::NIL
        );
        context.set(ident, value.clone());
        assert!(ast_vm::eval_module(&module, &mut context)?.total_eq(&value.into()));
        Ok(())
    }

    #[quickcheck]
    fn eval_table_lookup_on_nonexistent_key(key: LuaKey) -> Result<(), LuaError> {
        let module = lua_parser::module(
            "tbl = {}
            return tbl[key]",
        )?;
        let mut context = GlobalContext::new();
        context.set("key", LuaValue::from(key));

        let res = ast_vm::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::NIL);

        Ok(())
    }

    #[quickcheck]
    fn everything_that_is_not_a_table_cannot_be_indexed(
        value: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if value.is_table() {
            return Ok(TestResult::discard());
        }

        let module = lua_parser::module("return value[1]")?;
        let mut context = GlobalContext::new();
        context.set("value", value);
        let res = ast_vm::eval_module(&module, &mut context);
        assert_type_error!(TypeError::IsNotIndexable(_), res);

        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_table_lookup(value: LuaValue, key: LuaKey) -> Result<TestResult, LuaError> {
        if let LuaKey::Number(num) = key {
            if num.as_f64().is_nan() {
                return Ok(TestResult::discard());
            }
        }

        let module = lua_parser::module("return tbl[key]")?;
        let mut context = GlobalContext::new();
        let mut table = TableValue::new();
        table.set(key.clone(), value.clone());
        context.set("tbl", LuaValue::table(table));
        context.set("key", LuaValue::from(key));

        let res = ast_vm::eval_module(&module, &mut context)?.assert_single();
        assert!(res.total_eq(&value));

        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn accessing_property_can_only_be_done_on_a_table(
        value: LuaValue,
    ) -> Result<TestResult, LuaError> {
        if value.is_table() {
            return Ok(TestResult::discard());
        }

        let module = lua_parser::module("return value.foo")?;
        let mut context = GlobalContext::new();
        context.set("value", value);
        let res = ast_vm::eval_module(&module, &mut context);
        assert_type_error!(TypeError::CannotAccessProperty { .. }, res);
        Ok(TestResult::passed())
    }

    #[quickcheck]
    fn eval_property_access(value: LuaValue, property: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("return tbl.{}", property))?;
        let mut context = GlobalContext::new();
        let mut table = TableValue::new();
        let key = LuaKey::string(property);
        table.set(key, value.clone());
        context.set("tbl", LuaValue::table(table));

        let res = ast_vm::eval_module(&module, &mut context)?.assert_single();
        assert!(res.total_eq(&value));
        Ok(())
    }

    #[quickcheck]
    fn looking_up_nonexistent_property(property: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("tbl = {{}} return tbl.{}", property))?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::NIL);
        Ok(())
    }

    #[quickcheck]
    fn property_lookup_is_the_same_as_string_lookup(
        table: TableValue,
        prop: Ident,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("return tbl[\"{}\"], tbl.{}", prop, prop))?;
        let mut context = GlobalContext::new();
        context.set("tbl", LuaValue::table(table));
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.is_multiple_return());
        assert_eq!(res.len(), 2);
        assert!(res[0].total_eq(&res[1]));
        Ok(())
    }
}
