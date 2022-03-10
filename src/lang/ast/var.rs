use crate::{
    lang::{Eval, EvalContext, EvalContextExt, EvalError, LuaKey, LuaValue, TypeError},
    lex::Ident,
    syn::Var,
};

impl Eval for Var {
    type Return = LuaValue;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, EvalError>
    where
        Context: EvalContext + ?Sized,
    {
        match self {
            Self::Named(ident) => Ok(context.get(ident).clone()),
            Self::MemberLookup { from, value } => {
                let from = from.eval(context)?;
                let key = value.eval(context)?.first_value();
                member_lookup(from, key)
            }
            Self::PropertyAccess { from, property } => {
                let from = from.eval(context)?;
                property_access(from, property.clone())
            }
        }
        .map_err(EvalError::TypeError)
    }
}

fn member_lookup(value: LuaValue, key: LuaValue) -> Result<LuaValue, TypeError> {
    if let LuaValue::Table(table) = value {
        if let Some(key) = LuaKey::new(key) {
            Ok(table.get(&key))
        } else {
            Err(TypeError::NilLookup)
        }
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

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use crate::{
        error::LuaError,
        lang::{
            Eval, EvalContextExt, EvalError, GlobalContext, LuaKey, LuaValue, ReturnValue,
            TableValue, TypeError,
        },
        lex::Ident,
        syn::lua_parser,
    };

    #[quickcheck]
    fn eval_ident_on_global(value: LuaValue, ident: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("return {}", ident))?;
        let mut context = GlobalContext::new();
        assert_eq!(module.eval(&mut context)?, ReturnValue::Nil);
        context.set(ident, value.clone());
        assert!(module.eval(&mut context)?.total_eq(&value.into()));
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

        let res = module.eval(&mut context)?;
        assert_eq!(res, ReturnValue::Nil);

        Ok(())
    }

    #[test]
    fn looking_up_table_with_nil_results_in_an_type_error() -> Result<(), LuaError> {
        let module = lua_parser::module(
            "tbl = {}
            return tbl[nil]",
        )?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context);
        assert!(matches!(
            res,
            Err(EvalError::TypeError(TypeError::NilLookup))
        ));
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
        let res = module.eval(&mut context);
        assert!(matches!(
            res,
            Err(EvalError::TypeError(TypeError::IsNotIndexable(_)))
        ));

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

        let res = module.eval(&mut context)?.assert_single();
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
        let res = module.eval(&mut context);
        assert!(matches!(
            res,
            Err(EvalError::TypeError(TypeError::CannotAccessProperty { .. }))
        ));

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

        let res = module.eval(&mut context)?.assert_single();
        assert!(res.total_eq(&value));
        Ok(())
    }

    #[quickcheck]
    fn looking_up_nonexistent_property(property: Ident) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("tbl = {{}} return tbl.{}", property))?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?;
        assert_eq!(res, ReturnValue::Nil);
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
        let res = module.eval(&mut context)?;
        assert!(res.is_multiple_return());
        let values = res.unwrap_multiple_return();
        assert_eq!(values.len(), 2);
        assert!(values[0].total_eq(&values[1]));
        Ok(())
    }
}
