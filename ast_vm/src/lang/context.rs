use super::LuaValue;
use crate::opt;
use non_empty::NonEmptyVec;
use std::{collections::{hash_map::Entry, HashMap}, marker::PhantomData};

#[derive(Debug, Clone, Default)]
pub(crate) struct Scope(pub HashMap<String, LuaValue>);

// #[derive(Debug, Clone, Default)]
// struct GlobalScope {
//     scope: Scope,
//     global_nil: LuaValue,
// }

// impl GlobalScope {
//     pub fn get(&self, ident: impl AsRef<str>) -> &LuaValue {
//         self.scope.0.get(ident.as_ref()).unwrap_or(&self.global_nil)
//     }

//     pub fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
//         self.scope.0.insert(ident.into(), value);
//     }

//     pub fn contains(&self, ident: impl AsRef<str>) -> bool {
//         self.scope.0.contains_key(ident.as_ref())
//     }
// }

pub(crate) trait ScopeHolder {
    fn scopes(&self) -> &[Scope];
    fn scopes_mut(&mut self) -> &mut [Scope];
    fn global(&self) -> &Context;
    fn global_mut(&mut self) -> &mut Context;

    fn top_level_scope(&mut self) -> LocalScope<'_, Self>
    where
        Self: Sized;
    fn child_scope(&mut self, prev_scope: usize) -> LocalScope<'_, Self>
    where
        Self: Sized;
    fn declare_local(&mut self, scope: usize, ident: impl Into<String>, initial_value: LuaValue);
}

#[derive(Debug, Clone, Default)]
pub struct Context {
    // global_scope: GlobalScope,
    global_nil: LuaValue,
    local_scopes: NonEmptyVec<Scope>,

    // opt stuff
    pub globals: opt::GlobalValues,
    pub(crate) stack: Vec<LuaValue>,
    _not_send: PhantomData<*const ()>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, ident: impl AsRef<str>) -> &LuaValue {
        let id = self.globals.mapping.get(ident.as_ref());
        if let Some(&id) = id {
            return &self.globals.cells[id];
        }
        return &self.global_nil;
    }

    pub fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
        match self.globals.mapping.entry(ident.into()) {
            Entry::Occupied(entry) => {
                self.globals.cells[*entry.get()] = value;
            }
            Entry::Vacant(entry) => {
                let id = self.globals.cells.push(value);
                entry.insert(id);
            }
        }
    }

    pub fn contains(&self, ident: impl AsRef<str>) -> bool {
        self.globals.mapping.contains_key(ident.as_ref())
    }

    // pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
    //     self.into_iter()
    // }
}

impl ScopeHolder for Context {
    fn scopes(&self) -> &[Scope] {
        &self.local_scopes
    }

    fn scopes_mut(&mut self) -> &mut [Scope] {
        &mut self.local_scopes
    }

    fn global(&self) -> &Context {
        self
    }

    fn global_mut(&mut self) -> &mut Context {
        self
    }

    fn top_level_scope(&mut self) -> LocalScope<'_, Self>
    where
        Self: Sized,
    {
        LocalScope::new(self, 0)
    }

    fn child_scope(&mut self, prev_scope: usize) -> LocalScope<'_, Self>
    where
        Self: Sized,
    {
        let scope = prev_scope + 1;
        if scope == self.local_scopes.len().get() {
            self.local_scopes.push(Scope::default());
        }
        assert!(scope <= self.local_scopes.len().get());
        self.local_scopes[scope].0.clear();
        return LocalScope::new(self, scope);
    }

    fn declare_local(&mut self, scope: usize, ident: impl Into<String>, initial_value: LuaValue) {
        let key = ident.into();
        self.local_scopes[scope].0.insert(key, initial_value);
    }
}

pub(crate) struct FunctionContext<'a> {
    global: &'a mut Context,
    scopes: NonEmptyVec<Scope>,
}

impl<'a> FunctionContext<'a> {
    pub(crate) fn new(global: &'a mut Context) -> Self {
        Self {
            global,
            scopes: NonEmptyVec::default(),
        }
    }
}

impl<'a> ScopeHolder for FunctionContext<'a> {
    fn scopes(&self) -> &[Scope] {
        &self.scopes
    }

    fn scopes_mut(&mut self) -> &mut [Scope] {
        &mut self.scopes
    }

    fn global(&self) -> &Context {
        &self.global
    }

    fn global_mut(&mut self) -> &mut Context {
        &mut self.global
    }

    fn top_level_scope(&mut self) -> LocalScope<'_, Self>
    where
        Self: Sized,
    {
        LocalScope::new(self, 0)
    }

    fn child_scope(&mut self, prev_scope: usize) -> LocalScope<'_, Self>
    where
        Self: Sized,
    {
        let scope = prev_scope + 1;
        if scope == self.scopes.len().get() {
            self.scopes.push(Scope::default());
        }
        assert!(scope <= self.scopes.len().get());
        self.scopes[scope].0.clear();
        return LocalScope::new(self, scope);
    }

    fn declare_local(&mut self, scope: usize, ident: impl Into<String>, initial_value: LuaValue) {
        self.scopes[scope].0.insert(ident.into(), initial_value);
    }
}

pub(crate) struct LocalScope<'a, Parent> {
    parent: &'a mut Parent,
    scope: usize,
}

impl<'b, Parent: ScopeHolder> LocalScope<'b, Parent> {
    pub(crate) fn new(parent: &'b mut Parent, scope: usize) -> Self {
        Self { parent, scope }
    }

    pub(crate) fn get<'a>(&'a self, ident: impl AsRef<str>) -> &'a LuaValue {
        for scope in self.parent.scopes()[0..=(self.scope)].into_iter().rev() {
            if let Some(value) = scope.0.get(ident.as_ref()) {
                return value;
            }
        }
        self.parent.global().get(ident)
    }

    pub(crate) fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
        set_impl(self.parent, self.scope, ident.into(), value)
    }

    pub(crate) fn declare_local(&mut self, ident: impl Into<String>, initial_value: LuaValue) {
        self.parent.declare_local(self.scope, ident, initial_value)
    }

    pub(crate) fn child_scope(&mut self) -> LocalScope<'_, Parent> {
        self.parent.child_scope(self.scope)
    }

    pub(crate) fn global_mut(&mut self) -> &mut Context {
        self.parent.global_mut()
    }
}

fn set_impl(holder: &mut impl ScopeHolder, scope: usize, key: String, value: LuaValue) {
    let key = match holder.scopes_mut()[scope].0.entry(key) {
        Entry::Occupied(mut entry) => {
            entry.insert(value);
            return;
        }
        Entry::Vacant(entry) => entry.into_key(),
    };
    if scope == 0 {
        return holder.global_mut().set(key, value);
    }
    set_impl(holder, scope - 1, key, value)
}

// impl<'a> IntoIterator for &'a GlobalContext {
//     type Item = (&'a String, &'a LuaValue);

//     type IntoIter = std::collections::hash_map::Iter<'a, String, LuaValue>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.global_scope.scope.0.iter()
//     }
// }

// impl IntoIterator for GlobalContext {
//     type Item = (String, LuaValue);

//     type IntoIter = std::collections::hash_map::IntoIter<String, LuaValue>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.global_scope.scope.0.into_iter()
//     }
// }
