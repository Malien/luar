use crate::{lex::Ident, syn::Var};

use super::{EvalError, LocalScope, LuaKey, LuaValue, ScopeHolder, TypeError};

mod block;
pub(crate) use block::*;
mod expr;
pub(crate) use expr::*;
mod fn_call;
pub(crate) use fn_call::*;
mod fn_decl;
pub(crate) use fn_decl::*;
mod module;
pub use module::*;
mod ret;
pub(crate) use ret::*;
mod stmnt;
pub(crate) use stmnt::*;
mod table_constructor;
pub(crate) use table_constructor::*;
mod var;
pub(crate) use var::*;

mod tail_values;
pub use tail_values::*;

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
    .map_err(EvalError::TypeError)
}

fn assign_to_value_member(of: LuaValue, key: LuaValue, value: LuaValue) -> Result<(), TypeError> {
    if let LuaValue::Table(mut table) = of {
        if let Some(key) = LuaKey::new(key) {
            table.set(key, value);
            Ok(())
        } else {
            Err(TypeError::NilLookup)
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
