use std::collections::{hash_map::Entry, HashMap};

use non_empty::NonEmptyVec;

use super::LuaValue;

#[derive(Debug, Clone, Default)]
pub(crate) struct Scope(pub HashMap<String, LuaValue>);

struct GlobalScope {
    scope: Scope,
    global_nil: LuaValue,
}

impl GlobalScope {
    fn new() -> Self {
        Self {
            scope: Scope::default(),
            global_nil: LuaValue::Nil,
        }
    }

    pub fn get(&self, ident: impl AsRef<str>) -> &LuaValue {
        self.scope.0.get(ident.as_ref()).unwrap_or(&self.global_nil)
    }

    pub fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
        self.scope.0.insert(ident.into(), value);
    }

    pub fn contains(&self, ident: impl AsRef<str>) -> bool {
        self.scope.0.contains_key(ident.as_ref())
    }
}

pub(crate) trait ScopeHolder {
    fn scopes(&self) -> &[Scope];
    fn scopes_mut(&mut self) -> &mut [Scope];
    fn global(&self) -> &GlobalContext;
    fn global_mut(&mut self) -> &mut GlobalContext;

    fn top_level_scope(&mut self) -> LocalScope<'_, Self>
    where
        Self: Sized;
    fn child_scope(&mut self, prev_scope: usize) -> LocalScope<'_, Self>
    where
        Self: Sized;
    fn declare_local(&mut self, scope: usize, ident: impl Into<String>, initial_value: LuaValue);
}

pub struct GlobalContext {
    global_scope: GlobalScope,
    local_scopes: NonEmptyVec<Scope>,
}

impl GlobalContext {
    pub fn new() -> Self {
        Self {
            global_scope: GlobalScope::new(),
            local_scopes: NonEmptyVec::default(),
        }
    }

    pub fn get(&self, ident: impl AsRef<str>) -> &LuaValue {
        self.global_scope.get(ident)
    }

    pub fn set(&mut self, ident: impl Into<String>, value: LuaValue) {
        self.global_scope.set(ident, value)
    }

    pub fn contains(&self, ident: impl AsRef<str>) -> bool {
        self.global_scope.contains(ident)
    }
}

impl ScopeHolder for GlobalContext {
    fn scopes(&self) -> &[Scope] {
        &self.local_scopes
    }

    fn scopes_mut(&mut self) -> &mut [Scope] {
        &mut self.local_scopes
    }

    fn global(&self) -> &GlobalContext {
        self
    }

    fn global_mut(&mut self) -> &mut GlobalContext {
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
        if scope == self.local_scopes.len() {
            self.local_scopes.push(Scope::default());
        }
        assert!(scope <= self.local_scopes.len());
        self.local_scopes[scope].0.clear();
        return LocalScope::new(self, scope);
    }

    fn declare_local(&mut self, scope: usize, ident: impl Into<String>, initial_value: LuaValue) {
        let key = ident.into();
        if scope != 0 || !self.global_scope.contains(&key) {
            self.local_scopes[scope]
                .0
                .entry(key)
                .or_insert(initial_value);
        }
    }
}

pub(crate) struct FunctionContext<'a> {
    global: &'a mut GlobalContext,
    scopes: NonEmptyVec<Scope>,
}

impl<'a> FunctionContext<'a> {
    pub(crate) fn new(global: &'a mut GlobalContext) -> Self {
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

    fn global(&self) -> &GlobalContext {
        &self.global
    }

    fn global_mut(&mut self) -> &mut GlobalContext {
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
        if scope == self.scopes.len() {
            self.scopes.push(Scope::default());
        }
        assert!(scope <= self.scopes.len());
        self.scopes[scope].0.clear();
        return LocalScope::new(self, scope);
    }

    fn declare_local(&mut self, scope: usize, ident: impl Into<String>, initial_value: LuaValue) {
        self.scopes[scope]
            .0
            .entry(ident.into())
            .or_insert(initial_value);
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

    pub(crate) fn global_mut(&mut self) -> &mut GlobalContext {
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
