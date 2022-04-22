use super::{
    ids::{GlobalCellID, JmpLabel, LocalRegisterID, StringID},
    machine::GlobalValues,
    meta::LocalRegCount,
    ops::Instruction,
};
use crate::{ids::ArgumentRegisterID, keyed_vec::KeyedVec, meta};
use std::{collections::HashMap, num::NonZeroU16};

pub mod expr;
pub mod fn_call;
pub mod function;
pub mod module;
pub mod ret;
pub mod statement;
pub mod var;

pub use expr::*;
pub use fn_call::*;
pub use function::*;
pub use module::*;
pub use statement::*;
pub use var::*;

#[derive(Debug, Clone, Copy, Default)]
pub struct RegisterAllocator {
    total: LocalRegCount,
    in_use: LocalRegCount,
}

#[derive(Debug, Clone, Copy)]
pub struct LocalRegisterSpan {
    start: u16,
    count: u16,
}

pub struct NonEmptyLocalRegisterSpan {
    start: u16,
    count: NonZeroU16,
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

    pub fn alloc_dyn_nonzero(&mut self, count: NonZeroU16) -> NonEmptyLocalRegisterSpan {
        let start = self.in_use.d;
        self.in_use.d += count.get();
        self.total.d = std::cmp::max(self.total.d, self.in_use.d);
        NonEmptyLocalRegisterSpan { start, count }
    }

    pub fn free_dyn_count(&mut self, count: u16) {
        self.in_use.d -= count;
    }

    pub fn alloc_int(&mut self) -> LocalRegisterID {
        let reg_id = LocalRegisterID(self.in_use.i);
        self.in_use.i += 1;
        self.total.i = std::cmp::max(self.total.i, self.in_use.i);
        reg_id
    }

    pub fn free_int(&mut self) {
        self.in_use.i -= 1;
    }
}

impl LocalRegisterSpan {
    pub fn at(&self, index: u16) -> LocalRegisterID {
        self.try_at(index).unwrap()
    }

    pub fn try_at(&self, index: u16) -> Option<LocalRegisterID> {
        if index < self.count {
            Some(LocalRegisterID(self.start + index))
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a LocalRegisterSpan {
    type Item = LocalRegisterID;

    type IntoIter = std::iter::Map<std::ops::Range<u16>, fn(u16) -> LocalRegisterID>;

    fn into_iter(self) -> Self::IntoIter {
        (self.start..self.start + self.count)
            .into_iter()
            .map(LocalRegisterID)
    }
}

impl NonEmptyLocalRegisterSpan {
    pub fn at(&self, index: u16) -> LocalRegisterID {
        self.try_at(index).unwrap()
    }

    pub fn try_at(&self, index: u16) -> Option<LocalRegisterID> {
        if index < self.count.get() {
            Some(LocalRegisterID(self.start + index))
        } else {
            None
        }
    }

    pub fn last(&self) -> LocalRegisterID {
        LocalRegisterID(self.start + self.count.get() - 1)
    }
}

impl<'a> IntoIterator for &'a NonEmptyLocalRegisterSpan {
    type Item = LocalRegisterID;

    type IntoIter = std::iter::Map<std::ops::Range<u16>, fn(u16) -> LocalRegisterID>;

    fn into_iter(self) -> Self::IntoIter {
        (self.start..self.start + self.count.get())
            .into_iter()
            .map(LocalRegisterID)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LabelAllocator {
    current: u16,
    label_mapping: KeyedVec<JmpLabel, u32>,
}

impl LabelAllocator {
    pub fn alloc(&mut self) -> JmpLabel {
        let label = JmpLabel(self.current);
        self.current += 1;
        label
    }

    pub fn associate_label(&mut self, label: JmpLabel, instruction_position: u32) {
        self.label_mapping.accommodate_for_key(label, 0);
        self.label_mapping[label] = instruction_position;
    }

    pub fn into_mappings(self) -> KeyedVec<JmpLabel, u32> {
        self.label_mapping
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocalScope(HashMap<String, LocalRegisterID>);

#[derive(Debug, Clone, Default)]
pub struct ArgumentScope(HashMap<String, ArgumentRegisterID>);

#[derive(Debug, Clone, Copy)]
pub enum ReturnCountState {
    NotSpecified,
    Unbounded,
    MinBounded(NonZeroU16),
    Bounded { min: u16, max: NonZeroU16 },
    Constant(u16),
}

fn nonzero_max(x: NonZeroU16, y: u16) -> NonZeroU16 {
    // SAFETY: new value can never be less than previous non-zero value of x,
    // which means, since x cannot be zero, so can't the new value
    unsafe { NonZeroU16::new_unchecked(std::cmp::max(x.get(), y)) }
}

impl ReturnCountState {
    pub fn with_known_count(self, count: u16) -> ReturnCountState {
        use ReturnCountState::*;

        match self {
            NotSpecified => Constant(count),
            Unbounded => Unbounded,
            MinBounded(prev_min) => match NonZeroU16::new(count) {
                Some(count) => MinBounded(std::cmp::min(count, prev_min)),
                None => Unbounded,
            },
            Bounded { min, max } => Bounded {
                min: std::cmp::min(min, count),
                max: nonzero_max(max, count),
            },
            Constant(prev_count) if prev_count == count => Constant(count),
            Constant(prev_count) => Bounded {
                min: std::cmp::min(prev_count, count),
                // SAFETY: The only possibility of resulting value being 0 is if the both
                // prev_count and count are zero. It is not possible, since I've checked
                // that those values are not equal to one another
                max: unsafe { NonZeroU16::new_unchecked(std::cmp::max(prev_count, count)) },
            },
        }
    }

    pub fn update_known(&mut self, count: u16) {
        *self = self.with_known_count(count);
    }

    pub fn with_known_min_count(self, min_count: NonZeroU16) -> ReturnCountState {
        use ReturnCountState::*;

        match self {
            NotSpecified => MinBounded(min_count),
            Unbounded => Unbounded,
            MinBounded(prev_min) => MinBounded(std::cmp::min(prev_min, min_count)),
            Constant(min) | Bounded { min, .. } => match NonZeroU16::new(min) {
                Some(prev_min) => MinBounded(std::cmp::min(prev_min, min_count)),
                None => Unbounded,
            },
        }
    }

    pub fn update_known_min(&mut self, min_count: NonZeroU16) {
        *self = self.with_known_min_count(min_count);
    }

    pub fn with_unknown_count(self) -> ReturnCountState {
        ReturnCountState::Unbounded
    }

    pub fn update_unknown(&mut self) {
        *self = self.with_unknown_count();
    }

    pub fn into_return_count(self) -> Option<meta::ReturnCount> {
        match self {
            Self::NotSpecified => None,
            Self::Unbounded => Some(meta::ReturnCount::Unbounded),
            Self::MinBounded(min) => Some(meta::ReturnCount::MinBounded(min)),
            Self::Bounded { min, max } => Some(meta::ReturnCount::Bounded { min, max }),
            Self::Constant(count) => Some(meta::ReturnCount::Constant(count)),
        }
    }
}

impl Default for ReturnCountState {
    fn default() -> Self {
        Self::NotSpecified
    }
}

#[derive(Debug)]
pub struct FunctionCompilationState<'a> {
    global_values: &'a mut GlobalValues,
    reg_alloc: RegisterAllocator,
    label_alloc: LabelAllocator,
    strings: KeyedVec<StringID, String>,
    instructions: Vec<Instruction>,
    arguments: ArgumentScope,
    scope_vars: Vec<LocalScope>,
    return_count: ReturnCountState,
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
            return_count: Default::default(),
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
            return_count: Default::default(),
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

    pub fn strings(&mut self) -> &mut KeyedVec<StringID, String> {
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
        let instruction_position = self.instructions().len().try_into().unwrap();
        self.push_instr(Instruction::Label);
        self.func_state
            .label_alloc
            .associate_label(label, instruction_position);
    }

    pub fn return_count(&mut self) -> &mut ReturnCountState {
        &mut self.func_state.return_count
    }
}
