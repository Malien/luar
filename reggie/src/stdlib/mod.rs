use crate::{EvalError, LuaValue, OverloadRule, OverloadSet};

fn assert_none() -> Result<(), EvalError> {
    Err(EvalError::AssertionError)
}

fn assert_some(value: LuaValue) -> Result<(), EvalError> {
    if value.is_truthy() {
        Ok(())
    } else {
        Err(EvalError::AssertionError)
    }
}

pub fn assert() -> OverloadSet {
    OverloadSet::new(vec![
        OverloadRule::from(assert_none as fn() -> Result<(), EvalError>),
        OverloadRule::from(assert_some as fn(LuaValue) -> Result<(), EvalError>),
    ])
}
