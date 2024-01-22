use std::collections::HashMap;

use crate::{LuaValue, ids::GlobalCellID};
use keyed_vec::KeyedVec;

#[derive(Debug, Clone)]
pub struct GlobalValueCell {
    pub value: LuaValue,
    pub name: String,
}

impl GlobalValueCell {
    pub fn new(name: String) -> Self {
        Self {
            name,
            value: LuaValue::Nil,
        }
    }

    pub fn with_value(name: String, value: LuaValue) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Default)]
pub struct GlobalValues {
    // Maybe it's better to utilize some kind of a amortized linked-list-like
    // structure to provide reference stability, as it is more efficient for
    // JIT-ted code to reference globals by their stable pointer value
    cells: KeyedVec<GlobalCellID, GlobalValueCell>,
    mapping: HashMap<String, GlobalCellID>,
    global_nil: LuaValue,
}

impl GlobalValues {
    pub fn cell_for_name<I: Into<String> + AsRef<str>>(&mut self, ident: I) -> GlobalCellID {
        let name = ident.into();
        *self
            .mapping
            .entry(name.clone())
            .or_insert_with(|| self.cells.push(GlobalValueCell::new(name)))
    }

    pub fn set<I: Into<String> + AsRef<str>>(&mut self, ident: I, value: LuaValue) -> GlobalCellID {
        use std::collections::hash_map::Entry::*;
        let name = ident.into();
        match self.mapping.entry(name) {
            Occupied(entry) => {
                let id = *entry.get();
                self.cells[id].value = value;
                id
            }
            Vacant(entry) => {
                let name = entry.key().clone();
                *entry.insert(self.cells.push(GlobalValueCell::with_value(name, value)))
            }
        }
    }

    pub fn get<I: AsRef<str>>(&self, ident: I) -> &LuaValue {
        self.mapping
            .get(ident.as_ref())
            .map(|cell_id| self.value_of_cell(*cell_id))
            .unwrap_or(&self.global_nil)
    }

    pub fn value_of_cell(&self, cell_id: GlobalCellID) -> &LuaValue {
        &self.cells[cell_id].value
    }

    pub fn set_cell(&mut self, cell_id: GlobalCellID, value: LuaValue) {
        self.cells[cell_id].value = value;
    }

    pub fn global_nil(&self) -> &LuaValue {
        &self.global_nil
    }
}

impl<'a> IntoIterator for &'a GlobalValues {
    type Item = &'a GlobalValueCell;

    type IntoIter = std::slice::Iter<'a, GlobalValueCell>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.slice().iter()
    }
}

impl std::fmt::Display for GlobalValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "global {{")?;
        for (cell_id, cell) in &self.cells {
            writeln!(f, "\t{} -> {:?}", cell_id.0, cell.name)?;
        }
        writeln!(f, "}}")
    }
}
