use std::collections::HashMap;

use super::LuaValue;

pub trait EvalContext {
    fn get<'a>(&'a self, ident: impl AsRef<str>) -> &'a LuaValue;
    fn set(&mut self, ident: impl Into<String>, value: LuaValue);
}

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
    fn get<'a>(&'a self, ident: impl AsRef<str>) -> &'a LuaValue {
        self.values.get(ident.as_ref()).unwrap_or(&self.global_nil)
    }

    fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
        self.values.insert(ident.into(), value);
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
    fn get<'a>(&'a self, ident: impl AsRef<str>) -> &'a LuaValue {
        self.values
            .get(ident.as_ref())
            .unwrap_or_else(|| self.parent.get(ident))
    }

    fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
        let str: String = ident.into();
        if self.values.contains_key(&str) {
            self.values.insert(str, value);
        } else {
            self.parent.set(str, value);
        }
    }
}

impl<'a, Parent> LocalContext<'a, Parent> {
    pub fn new(parent: &'a mut Parent) -> Self {
        Self {
            values: HashMap::new(),
            parent
        }
    }
}
