use std::collections::HashMap;

use super::LuaValue;

pub trait EvalContext {
    fn get_str<'a>(&'a self, ident: &str) -> &'a LuaValue;
    fn set_str(&mut self, ident: String, value: LuaValue);
    fn as_dyn(&self) -> &'_ dyn EvalContext;
    fn as_dyn_mut(&mut self) -> &'_ mut dyn EvalContext;
    fn declare_local(&mut self, ident: String, initial_value: LuaValue);
}

pub trait EvalContextExt: EvalContext {
    fn get<'a>(&'a self, ident: impl AsRef<str>) -> &'a LuaValue {
        self.get_str(ident.as_ref())
    }
    fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
        self.set_str(ident.into(), value)
    }
}

impl<T: EvalContext + ?Sized> EvalContextExt for T {}

pub struct GlobalContext {
    values: HashMap<String, LuaValue>,
    global_nil: LuaValue,
}

impl GlobalContext {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            global_nil: LuaValue::Nil,
        }
    }
}

impl EvalContext for GlobalContext {
    fn get_str<'a>(&'a self, ident: &str) -> &'a LuaValue {
        self.values.get(ident).unwrap_or(&self.global_nil)
    }
    fn set_str(&mut self, ident: String, value: LuaValue) {
        self.values.insert(ident, value);
    }

    fn as_dyn(&self) -> &'_ dyn EvalContext {
        self
    }
    fn as_dyn_mut(&mut self) -> &'_ mut dyn EvalContext {
        self
    }

    fn declare_local(&mut self, ident: String, initial_value: LuaValue) {
        self.values
            .entry(ident.to_string())
            .or_insert(initial_value);
    }
}

pub struct LocalContext<'a, Parent> {
    values: HashMap<String, LuaValue>,
    parent: &'a mut Parent,
}

impl<'b, Parent> EvalContext for LocalContext<'b, Parent>
where
    Parent: EvalContext,
{
    fn get_str<'a>(&'a self, ident: &str) -> &'a LuaValue {
        self.values
            .get(ident)
            .unwrap_or_else(|| self.parent.get_str(ident))
    }

    fn set_str(&mut self, ident: String, value: LuaValue) {
        if self.values.contains_key(&ident) {
            self.values.insert(ident, value);
        } else {
            self.parent.set_str(ident, value);
        }
    }

    fn as_dyn(&self) -> &'_ dyn EvalContext {
        self
    }
    fn as_dyn_mut(&mut self) -> &'_ mut dyn EvalContext {
        self
    }

    fn declare_local(&mut self, ident: String, initial_value: LuaValue) {
        self.values
            .entry(ident)
            .or_insert(initial_value);
    }
}

impl<'a, Parent> LocalContext<'a, Parent> {
    pub fn new(parent: &'a mut Parent) -> Self {
        Self {
            values: HashMap::new(),
            parent,
        }
    }
}
