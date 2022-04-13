use std::collections::HashMap;

use crate::lang::{LuaFunction, LuaValue, TableRef};

use super::{
    ids::{BlockID, GlobalCellID},
    meta::CodeMeta,
    ops::Instruction,
};

const ARG_REG_COUNT: usize = 16;

pub struct ArgumentRegisters {
    pub f: [f64; ARG_REG_COUNT],
    pub i: [i32; ARG_REG_COUNT],
    pub s: [Option<String>; ARG_REG_COUNT],
    pub t: [Option<TableRef>; ARG_REG_COUNT],
    pub c: [Option<LuaFunction>; ARG_REG_COUNT],
    pub d: [LuaValue; ARG_REG_COUNT],
}

pub struct Accumulators {
    pub f: f64,
    pub i: i32,
    pub s: Option<String>,
    pub t: Option<TableRef>,
    pub c: Option<LuaFunction>,
    pub d: LuaValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualityFlag {
    NE,
    EQ,
}

impl EqualityFlag {
    pub fn from_bool(v: bool) -> Self {
        match v {
            true => Self::EQ,
            false => Self::NE
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderingFlag {
    LT,
    GT,
}

pub enum TypeTestResult {
    Nil,
    Float,
    Int,
    String,
    Function,
    Table,
    Userdata,
}

#[derive(Debug, Clone)]
pub struct GlobalValueCell {
    value: LuaValue,
}

impl GlobalValueCell {
    pub fn with_value(value: LuaValue) -> Self {
        Self { value }
    }
}

impl Default for GlobalValueCell {
    fn default() -> Self {
        Self {
            value: LuaValue::Nil,
        }
    }
}

#[derive(Debug, Default)]
pub struct GlobalValues {
    // Maybe it's better to utilize some kind of a amortized linked-list-like
    // structure to provide reference stability, as it is more efficient for
    // JIT-ted code to reference globals by their stable pointer value
    cells: Vec<GlobalValueCell>,
    mapping: HashMap<String, GlobalCellID>,
}

fn alloc_cell(cells: &mut Vec<GlobalValueCell>, cell: GlobalValueCell) -> GlobalCellID {
    let idx = cells.len();
    cells.push(cell);
    GlobalCellID(idx.try_into().unwrap())
}

impl GlobalValues {
    pub fn cell_for_name<I: Into<String> + AsRef<str>>(&mut self, ident: I) -> GlobalCellID {
        *self
            .mapping
            .entry(ident.into())
            .or_insert_with(|| alloc_cell(&mut self.cells, GlobalValueCell::default()))
    }

    pub fn set<I: Into<String> + AsRef<str>>(&mut self, ident: I, value: LuaValue) -> GlobalCellID {
        use std::collections::hash_map::Entry::*;
        match self.mapping.entry(ident.into()) {
            Occupied(entry) => {
                let id = *entry.get();
                self.cells[id.0 as usize].value = value;
                id
            }
            Vacant(entry) => *entry.insert(alloc_cell(
                &mut self.cells,
                GlobalValueCell::with_value(value),
            )),
        }
    }

    pub fn value_of_cell(&self, cell_id: GlobalCellID) -> &LuaValue {
        &self.cells[cell_id.0 as usize].value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramCounter {
    pub block: BlockID,
    pub position: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub meta: CodeMeta,
    pub instructions: Vec<Instruction>,
}
pub struct CodeBlocks(Vec<CodeBlock>);

impl std::ops::Index<BlockID> for CodeBlocks {
    type Output = CodeBlock;

    fn index(&self, index: BlockID) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl std::ops::IndexMut<BlockID> for CodeBlocks {
    fn index_mut(&mut self, index: BlockID) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

impl CodeBlocks {
    pub fn add(&mut self, code_block: CodeBlock) -> BlockID {
        let id = self.0.len().try_into().unwrap();
        self.0.push(code_block);
        BlockID(id)
    }
}

impl Extend<CodeBlock> for CodeBlocks {
    fn extend<T: IntoIterator<Item = CodeBlock>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

pub struct LocalValues {
    pub f: Vec<f64>,
    pub i: Vec<i32>,
    pub s: Vec<Option<String>>,
    pub t: Vec<Option<TableRef>>,
    pub c: Vec<Option<LuaFunction>>,
    pub d: Vec<LuaValue>,
}

impl LocalValues {
    pub fn new(meta: &CodeMeta) -> Self {
        Self {
            f: vec![0.0; meta.local_count.f as usize],
            i: vec![0; meta.local_count.i as usize],
            s: vec![None; meta.local_count.s as usize],
            t: vec![None; meta.local_count.t as usize],
            c: vec![None; meta.local_count.c as usize],
            d: vec![LuaValue::Nil; meta.local_count.d as usize],
        }
    }
}

pub struct StackFrame {
    pub return_addr: ProgramCounter,
    pub local_values: LocalValues,
}

impl StackFrame {
    pub fn new(meta: &CodeMeta, return_addr: ProgramCounter) -> Self {
        StackFrame {
            return_addr,
            local_values: LocalValues::new(meta),
        }
    }
}

pub struct Machine {
    pub accumulators: Accumulators,
    pub program_counter: ProgramCounter,
    pub value_count: u32,
    pub equality_flag: EqualityFlag,
    pub ordering_flag: OrderingFlag,
    pub type_test_result: TypeTestResult,
    pub argument_registers: ArgumentRegisters,
    pub global_values: GlobalValues,
    pub code_blocks: CodeBlocks,
    pub stack: Vec<StackFrame>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            accumulators: Accumulators {
                f: 0.0,
                i: 0,
                s: None,
                t: None,
                c: None,
                d: LuaValue::Nil,
            },
            program_counter: ProgramCounter {
                block: BlockID(0),
                position: 0,
            },
            value_count: 0,
            equality_flag: EqualityFlag::NE,
            ordering_flag: OrderingFlag::LT,
            type_test_result: TypeTestResult::Nil,
            argument_registers: ArgumentRegisters {
                f: [0.0; ARG_REG_COUNT],
                i: [0; ARG_REG_COUNT],
                s: [(); ARG_REG_COUNT].map(|_| None),
                t: [(); ARG_REG_COUNT].map(|_| None),
                c: [(); ARG_REG_COUNT].map(|_| None),
                d: [(); ARG_REG_COUNT].map(|_| LuaValue::Nil),
            },
            global_values: GlobalValues::default(),
            code_blocks: CodeBlocks(vec![]),
            stack: vec![],
        }
    }
}
