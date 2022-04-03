use std::collections::HashMap;

use crate::reggie::ids::ArgumentRegisterID;

use super::{meta::LocalRegCount, ids::{LocalRegisterID, GlobalCellID, StringID}, ops::Instruction, machine::GlobalValues};

pub mod expr;
pub mod func;
pub mod module;

pub use expr::*;
pub use func::*;
pub use module::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct RegisterAllocator {
    total: LocalRegCount,
    in_use: LocalRegCount,
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
}

#[derive(Debug, Clone, Default)]
pub struct LocalScope(HashMap<String, LocalRegisterID>);

#[derive(Debug, Clone, Default)]
pub struct ArgumentScope(HashMap<String, ArgumentRegisterID>);

#[derive(Debug)]
pub struct FunctionCompilationState<'a> {
    global_values: &'a mut GlobalValues,
    alloc: RegisterAllocator,
    strings: Vec<String>,
    instructions: Vec<Instruction>,
    arguments: ArgumentScope,
    scope_vars: Vec<LocalScope>,
}

impl<'a> FunctionCompilationState<'a> {
    pub fn new(global_values: &'a mut GlobalValues) -> Self {
        Self {
            global_values,
            alloc: Default::default(),
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
            alloc: Default::default(),
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
}

#[derive(Debug)]
pub struct LocalFnCompState<'a, 'b> {
    func_state: &'a mut FunctionCompilationState<'b>,
    scope: usize,
}

pub enum VarLookup {
    Argument(ArgumentRegisterID),
    Local(LocalRegisterID),
    GlobalCell(GlobalCellID),
}

impl<'a, 'b> LocalFnCompState<'a, 'b> {
    pub fn reg(&mut self) -> &mut RegisterAllocator {
        &mut self.func_state.alloc
    }

    pub fn strings(&mut self) -> &mut Vec<String> {
        &mut self.func_state.strings
    }

    pub fn instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.func_state.instructions
    }

    pub fn push_instr(&mut self, instr: Instruction) {
        self.func_state.instructions.push(instr)
    }

    pub fn alloc_string(&mut self, str: String) -> StringID {
        let str_idx = self.strings().len();
        self.strings().push(str);
        StringID(str_idx.try_into().unwrap())
    }

    pub fn lookup_var(&mut self, ident: &str) -> VarLookup {
        let local_reg = self.func_state.scope_vars[..=(self.scope)]
            .into_iter()
            .rev()
            .find_map(|scope| scope.0.get(ident));

        if let Some(register) = local_reg {
            VarLookup::Local(*register)
        } else if let Some(register) = self.func_state.arguments.0.get(ident) {
            VarLookup::Argument(*register)
        } else {
            VarLookup::GlobalCell(self.func_state.global_values.cell_for_name(ident))
        }
    }

    pub fn define_local(&mut self, ident: String, location: LocalRegisterID) {
        self.func_state.scope_vars[self.scope]
            .0
            .insert(ident, location);
    }

    pub fn inner_scope(self) -> Self {
        if self.func_state.scope_vars.len() == self.scope - 1 {
            let scope = LocalScope::default();
            self.func_state.scope_vars.push(scope);
        } else {
            self.func_state.scope_vars[self.scope + 1].0.clear();
        }
        Self {
            scope: self.scope + 1,
            ..self
        }
    }

    pub fn new(func_state: &'a mut FunctionCompilationState<'b>) -> Self {
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
}
