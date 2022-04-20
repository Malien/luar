use std::collections::HashMap;

use crate::ids::ArgumentRegisterID;

use super::{
    ids::{GlobalCellID, JmpLabel, LocalRegisterID, StringID},
    machine::GlobalValues,
    meta::LocalRegCount,
    ops::Instruction,
};

pub mod expr;
pub mod fn_call;
pub mod func;
pub mod module;
pub mod ret;
pub mod statement;
pub mod var;

pub use expr::*;
pub use fn_call::*;
pub use func::*;
pub use module::*;
pub use statement::*;
pub use var::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct RegisterAllocator {
    total: LocalRegCount,
    in_use: LocalRegCount,
}

pub struct LocalRegisterSpan {
    start: u16,
    count: u16,
}

impl RegisterAllocator {
    pub fn into_used_register_count(self) -> LocalRegCount {
        self.total
    }

    pub fn alloc_dyn(&mut self) -> LocalRegisterID {
        let reg_id = LocalRegisterID(self.in_use.d);
        self.in_use.d += 1;
        self.total.d = std::cmp::max(self.total.d, self.in_use.d);
        reg_id
    }

    pub fn free_dyn(&mut self) {
        self.in_use.d -= 1;
    }

    pub fn alloc_dyn_count(&mut self, count: u16) -> LocalRegisterSpan {
        let start = self.in_use.d;
        self.in_use.d += count;
        self.total.d = std::cmp::max(self.total.d, self.in_use.d);
        LocalRegisterSpan { start, count }
    }

    pub fn free_dyn_count(&mut self, count: u16) {
        self.in_use.d -= count;
    }
}

impl LocalRegisterSpan {
    fn at(&self, index: u16) -> LocalRegisterID {
        assert!(index < self.count);
        LocalRegisterID(self.start + index)
    }
}

impl<'a> IntoIterator for &'a LocalRegisterSpan {
    type Item = LocalRegisterID;

    type IntoIter = std::iter::Map<std::ops::Range<u16>, fn (u16) -> LocalRegisterID>;

    fn into_iter(self) -> Self::IntoIter {
        (self.start..self.start + self.count).into_iter().map(LocalRegisterID)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LabelAllocator {
    current: u16,
    label_mapping: Vec<usize>,
}

impl LabelAllocator {
    pub fn alloc(&mut self) -> JmpLabel {
        let label = JmpLabel(self.current);
        self.current += 1;
        label
    }

    pub fn associate_label(&mut self, label: JmpLabel, instruction_position: usize) {
        self.label_mapping.resize(label.0 as usize + 1, 0);
        self.label_mapping[label.0 as usize] = instruction_position;
    }

    pub fn into_mappings(self) -> Vec<usize> {
        self.label_mapping
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocalScope(HashMap<String, LocalRegisterID>);

#[derive(Debug, Clone, Default)]
pub struct ArgumentScope(HashMap<String, ArgumentRegisterID>);

#[derive(Debug)]
pub struct FunctionCompilationState<'a> {
    global_values: &'a mut GlobalValues,
    reg_alloc: RegisterAllocator,
    label_alloc: LabelAllocator,
    strings: Vec<String>,
    instructions: Vec<Instruction>,
    arguments: ArgumentScope,
    scope_vars: Vec<LocalScope>,
}

impl<'a> FunctionCompilationState<'a> {
    pub fn new(global_values: &'a mut GlobalValues) -> Self {
        Self {
            global_values,
            reg_alloc: Default::default(),
            label_alloc: Default::default(),
            strings: Default::default(),
            instructions: Default::default(),
            arguments: Default::default(),
            scope_vars: Default::default(),
        }
    }

    pub fn with_args(
        args: impl IntoIterator<Item = impl Into<String>>,
        global_values: &'a mut GlobalValues,
    ) -> Self {
        Self {
            global_values,
            reg_alloc: Default::default(),
            label_alloc: Default::default(),
            strings: Default::default(),
            instructions: Default::default(),
            arguments: ArgumentScope(
                args.into_iter()
                    .map(Into::into)
                    .zip((0..).into_iter().map(ArgumentRegisterID))
                    .collect(),
            ),
            scope_vars: Default::default(),
        }
    }

    pub fn global_values(&mut self) -> &mut GlobalValues {
        self.global_values
    }
}

#[derive(Debug)]
pub struct LocalScopeCompilationState<'a, 'b> {
    func_state: &'a mut FunctionCompilationState<'b>,
    scope: usize,
}

pub enum VarLookup {
    Argument(ArgumentRegisterID),
    Local(LocalRegisterID),
    GlobalCell(GlobalCellID),
}

impl<'a, 'b> LocalScopeCompilationState<'a, 'b> {
    pub fn reg(&mut self) -> &mut RegisterAllocator {
        &mut self.func_state.reg_alloc
    }

    pub fn strings(&mut self) -> &mut Vec<String> {
        &mut self.func_state.strings
    }

    pub fn push_instr(&mut self, instr: Instruction) {
        self.func_state.instructions.push(instr)
    }

    pub fn alloc_string(&mut self, str: String) -> StringID {
        let str_idx = self.strings().len();
        self.strings().push(str);
        StringID(str_idx.try_into().unwrap())
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.func_state.instructions
    }

    pub fn lookup_var(&mut self, ident: impl AsRef<str>) -> VarLookup {
        let local_reg = self.func_state.scope_vars[..=(self.scope)]
            .into_iter()
            .rev()
            .find_map(|scope| scope.0.get(ident.as_ref()));

        if let Some(register) = local_reg {
            VarLookup::Local(*register)
        } else if let Some(register) = self.func_state.arguments.0.get(ident.as_ref()) {
            VarLookup::Argument(*register)
        } else {
            VarLookup::GlobalCell(self.func_state.global_values.cell_for_name(ident.as_ref()))
        }
    }

    pub fn define_local(&mut self, ident: String, location: LocalRegisterID) {
        self.func_state.scope_vars[self.scope]
            .0
            .insert(ident, location);
    }

    pub fn inner_scope<'c>(&'c mut self) -> LocalScopeCompilationState<'c, 'b> {
        if self.func_state.scope_vars.len() - 1 == self.scope {
            let scope = LocalScope::default();
            self.func_state.scope_vars.push(scope);
        } else {
            self.func_state.scope_vars[self.scope + 1].0.clear();
        }
        Self {
            scope: self.scope + 1,
            // SAFETY: Oh my fucking god! Just shut up! I can't, for the life of me, figure out
            // how to do these lifetimes appropriately. So fuck borrow checker, I'll do my own thing.
            // By it's fucking definition 'c is smaller than 'a or 'b, or otherwise there couldn't
            // be am object from which I could take &'c!
            func_state: unsafe { &mut *(self.func_state as *mut FunctionCompilationState<'b>) },
        }
    }

    pub fn new(
        func_state: &'a mut FunctionCompilationState<'b>,
    ) -> LocalScopeCompilationState<'a, 'b> {
        if func_state.scope_vars.len() == 0 {
            let scope = LocalScope::default();
            func_state.scope_vars.push(scope);
        } else {
            func_state.scope_vars[0].0.clear();
        }
        Self {
            func_state,
            scope: 0,
        }
    }

    pub fn global_values(&mut self) -> &mut GlobalValues {
        self.func_state.global_values()
    }

    pub fn alloc_label(&mut self) -> JmpLabel {
        self.func_state.label_alloc.alloc()
    }

    pub fn push_label(&mut self, label: JmpLabel) {
        let instruction_position = self.func_state.instructions.len();
        self.push_instr(Instruction::Label);
        self.func_state
            .label_alloc
            .associate_label(label, instruction_position);
    }
}
