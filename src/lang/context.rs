use std::collections::HashMap;

use super::LuaValue;

pub struct EvalContext {
    values: HashMap<String, LuaValue>,
    global_nil: LuaValue,
}

impl EvalContext {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            global_nil: LuaValue::Nil,
        }
    }

    pub fn get<'a>(&'a self, ident: impl AsRef<str>) -> &'a LuaValue {
        self.values.get(ident.as_ref()).unwrap_or(&self.global_nil)
    }

    pub fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
        self.values.insert(ident.into(), value);
    }
}