use crate::lang::{LocalScope, LuaValue, ScopeHolder};
use luar_lex::Ident;
use luar_syn::Declaration;

use super::assignment::assignment_values;

pub(crate) fn eval_decl(
    decl: &Declaration,
    scope: &mut LocalScope<impl ScopeHolder>,
) -> Result<(), crate::EvalError> {
    let Declaration {
        names,
        initial_values,
    } = decl;

    assignment_values(scope, initial_values)
        .map(|values| multiple_local_assignment(scope, names.clone(), values))
}

fn multiple_local_assignment(
    scope: &mut LocalScope<impl ScopeHolder>,
    names: impl IntoIterator<Item = Ident>,
    values: impl Iterator<Item = LuaValue>,
) {
    for (name, value) in names.into_iter().zip(values) {
        scope.declare_local(name, value);
    }
}

#[cfg(test)]
mod test {
    use crate as ast_vm;
    use crate::{
        lang::{Context, LuaValue, ReturnValue},
        LuaError,
    };
    use luar_lex::Ident;
    use luar_syn::lua_parser;

    #[quickcheck]
    fn local_decl_does_not_behave_like_global_assignment_in_global_scope(
        ident: Ident,
        value: LuaValue,
    ) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!("local {} = value", ident))?;
        let mut context = Context::new();
        context.set("value", value.clone());
        ast_vm::eval_module(&module, &mut context)?;
        assert_eq!(context.get(&ident), &LuaValue::Nil);
        Ok(())
    }

    #[quickcheck]
    fn redeclaring_local_overrides_the_variable(ident: Ident, value: LuaValue) -> Result<(), LuaError> {
        let module = lua_parser::module(&format!(
            "local {} = value
            local {}
            return {}",
            ident, ident, ident
        ))?;
        let mut context = Context::new();
        context.set("value", value.clone());
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert_eq!(res.assert_single(), LuaValue::Nil);
        Ok(())
    }

    #[quickcheck]
    fn redeclaring_local_with_new_value_sets_the_value(
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
        let mut context = Context::new();
        context.set("value1", value1);
        context.set("value2", value2.clone());
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert!(res.assert_single().total_eq(&value2));
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
        let mut context = Context::new();
        context.set("value1", value1.clone());
        context.set("value2", value2);
        ast_vm::eval_module(&module, &mut context)?;
        assert!(context.get(&ident).total_eq(&value1));
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
        let mut context = Context::new();
        context.set("value1", value1);
        context.set("value2", value2);
        ast_vm::eval_module(&module, &mut context)?;
        assert!(context.get(&ident).is_nil());
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
        let mut context = Context::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::NIL);
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
        let mut context = Context::new();
        let res = ast_vm::eval_module(&module, &mut context)?;
        assert_eq!(res, ReturnValue::NIL);
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
        let res = ast_vm::eval_module(&module, &mut Context::new())?;
        assert_eq!(res, ReturnValue::NIL);
        Ok(())
    }
}
