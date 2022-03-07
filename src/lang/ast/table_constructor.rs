use crate::{
    lang::{Eval, LuaKey, TableValue},
    syn::TableConstructor,
};

impl Eval for TableConstructor {
    type Return = TableValue;

    fn eval<Context>(&self, context: &mut Context) -> Result<Self::Return, crate::lang::EvalError>
    where
        Context: crate::lang::EvalContext + ?Sized,
    {
        let Self { lfield, ffield } = self;
        let mut table = TableValue::new();
        for (value, idx) in lfield.into_iter().zip(1..) {
            let key = LuaKey::number(idx);
            let value = value.eval(context)?.first_value();
            table.set(key, value);
        }
        for (ident, expr) in ffield {
            let key = LuaKey::string(ident.as_ref());
            let value = expr.eval(context)?.first_value();
            table.set(key, value);
        }

        Ok(table)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        error::LuaError,
        lang::{Eval, EvalContextExt, GlobalContext, LuaKey, LuaValue, TableValue},
        lex::Ident,
        syn::{string_parser, Expression, TableConstructor, Var},
        test_util::vec_of_idents,
        util::NonEmptyVec,
    };

    #[test]
    fn empty_table_constructor_creates_empty_table() -> Result<(), LuaError> {
        let module = string_parser::module("return {}")?;
        let mut context = GlobalContext::new();
        let res = module.eval(&mut context)?.assert_single();
        assert!(res.is_table());
        assert!(res.unwrap_table().is_empty());
        Ok(())
    }

    #[quickcheck]
    fn list_table_constructor_creates_table_with_integer_indices_from_one_upwards(
        values: NonEmptyVec<LuaValue>,
    ) -> Result<(), LuaError> {
        let idents = vec_of_idents(values.len(), "value");

        let module = TableConstructor {
            lfield: idents
                .iter()
                .cloned()
                .map(Var::Named)
                .map(Expression::Variable)
                .collect(),
            ffield: vec![],
        };

        let mut context = GlobalContext::new();
        for (value, ident) in values.iter().cloned().zip(idents) {
            context.set(ident, value);
        }

        let res = module.eval(&mut context)?;

        let mut expected = TableValue::new();
        for (value, idx) in values.into_iter().zip(1..) {
            expected.set(LuaKey::number(idx), value)
        }

        assert!(res.total_eq(&expected));

        Ok(())
    }

    #[quickcheck]
    fn key_value_pairs_constructor_creates_table_with_corresponding_key_value_association(
        values: NonEmptyVec<(Ident, LuaValue)>,
    ) -> Result<(), LuaError> {
        let idents = vec_of_idents(
            values.len(),
            "please_do_not_collide_with_autogenerated_idents",
        );

        let module = TableConstructor {
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

        let mut context = GlobalContext::new();
        for (value, ident) in values.iter().map(|(_, value)| value).cloned().zip(idents) {
            context.set(ident, value);
        }

        let res = module.eval(&mut context)?;

        let mut expected = TableValue::new();
        for (key, value) in values {
            expected.set(LuaKey::String(key.into()), value)
        }

        assert!(res.total_eq(&expected));

        Ok(())
    }
}
