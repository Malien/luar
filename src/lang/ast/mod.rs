use crate::{lex::Ident, syn::Var};

use super::{Eval, EvalContext, EvalContextExt, EvalError, LuaKey, LuaValue, TypeError};

mod block;
mod expr;
mod fn_call;
mod fn_decl;
mod module;
mod ret;
mod stmnt;
mod table_constructor;
mod var;

mod tail_values;
pub use tail_values::*;

pub fn assign_to_var<Context>(
    context: &mut Context,
    var: &Var,
    value: LuaValue,
) -> Result<(), EvalError>
where
    Context: EvalContext + ?Sized,
{
    match var {
        Var::Named(ident) => Ok(context.set(ident.clone(), value)),
        Var::MemberLookup { from, value: key } => {
            let from = from.eval(context)?;
            let key = key.eval(context)?.first_value();
            assign_to_value_member(from, key, value)
        }
        Var::PropertyAccess { from, property } => {
            let from = from.eval(context)?;
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
