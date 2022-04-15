use crate::{
    lang::{LocalScope, LuaKey, ScopeHolder, TableValue},
    syn::TableConstructor,
};

use super::eval_expr;

pub(crate) fn eval_tbl_constructor(
    tbl: &TableConstructor,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<TableValue, crate::lang::EvalError> {
    let TableConstructor { lfield, ffield } = tbl;
    let mut table = TableValue::new();
    for (value, idx) in lfield.into_iter().zip(1..) {
        let key = LuaKey::number(idx);
        let value = eval_expr(value, scope)?.first_value();
        table.set(key, value);
    }
    for (ident, expr) in ffield {
        let key = LuaKey::string(ident.as_ref());
        let value = eval_expr(expr, scope)?.first_value();
        table.set(key, value);
    }

    Ok(table)
}

#[cfg(test)]
mod test {
    use luar_lex::Ident;
    use non_empty::NonEmptyVec;
    use test_util::vec_of_idents;

    use crate::{
        ast_vm::{self, expr::table_constructor::eval_tbl_constructor},
        error::LuaError,
        lang::{GlobalContext, LuaKey, LuaValue, ScopeHolder, TableValue},
        syn::{lua_parser, Expression, TableConstructor, Var},
    };

    #[test]
    fn empty_table_constructor_creates_empty_table() -> Result<(), LuaError> {
        let module = lua_parser::module("return {}")?;
        let mut context = GlobalContext::new();
        let res = ast_vm::eval_module(&module, &mut context)?.assert_single();
        assert!(res.is_table());
        assert!(res.unwrap_table().is_empty());
        Ok(())
    }

    #[quickcheck]
    fn list_table_constructor_creates_table_with_integer_indices_from_one_upwards(
        values: NonEmptyVec<LuaValue>,
    ) -> Result<(), LuaError> {
        let idents = vec_of_idents(values.len(), "value");

        let tbl = TableConstructor {
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

        let res = eval_tbl_constructor(&tbl, &mut context.top_level_scope())?;

        let mut expected = TableValue::new();
        for (value, idx) in values.into_iter().zip(1usize..) {
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

        let mut context = GlobalContext::new();
        for (value, ident) in values.iter().map(|(_, value)| value).cloned().zip(idents) {
            context.set(ident, value);
        }

        let res = eval_tbl_constructor(&tbl, &mut context.top_level_scope())?;

        let mut expected = TableValue::new();
        for (key, value) in values {
            expected.set(LuaKey::String(key.into()), value)
        }

        assert!(res.total_eq(&expected));

        Ok(())
    }
}
