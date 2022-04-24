use crate::{
    compiler::CompiledModule,
    global_values::GlobalValues,
    ids::{BlockID, LocalBlockID, ModuleID},
    keyed_vec::{keyed_vec, KeyedVec},
    meta::CodeMeta,
    ops::Instruction,
    LuaValue,
};

// const ARG_REG_COUNT: usize = 16;
// TODO: Implement ExtR in order to make argument register more likely to be in cache(?)
const ARG_REG_COUNT: usize = 32;

pub struct ArgumentRegisters {
    pub f: [f64; ARG_REG_COUNT],
    pub i: [i32; ARG_REG_COUNT],
    pub s: [Option<String>; ARG_REG_COUNT],
    pub d: [LuaValue; ARG_REG_COUNT],
}

pub struct Accumulators {
    pub f: f64,
    pub i: i32,
    pub s: Option<String>,
    pub c: BlockID,
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
            false => Self::NE,
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

impl std::fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} -> {}", self.meta.arg_count, self.meta.return_count)?;
        let lc = &self.meta.local_count;
        if lc.d != 0 || lc.i != 0 || lc.f != 0 || lc.s != 0 || lc.c != 0 {
            writeln!(f, "locals {{")?;
            if self.meta.local_count.d != 0 {
                writeln!(f, "\tD: {}", self.meta.local_count.d)?;
            }
            if self.meta.local_count.i != 0 {
                writeln!(f, "\tI: {}", self.meta.local_count.i)?;
            }
            if self.meta.local_count.f != 0 {
                writeln!(f, "\tF: {}", self.meta.local_count.f)?;
            }
            if self.meta.local_count.s != 0 {
                writeln!(f, "\tS: {}", self.meta.local_count.s)?;
            }
            if self.meta.local_count.c != 0 {
                writeln!(f, "\tC: {}", self.meta.local_count.c)?;
            }
            writeln!(f, "}}")?;
        }
        if !self.meta.const_strings.is_empty() {
            writeln!(f, "strings {{")?;
            for (string_id, string) in &self.meta.const_strings {
                writeln!(f, "\t{} -> {}", string_id.0, string)?;
            }
            writeln!(f, "}}")?;
        }
        if !self.meta.label_mappings.is_empty() {
            writeln!(f, "labels {{")?;
            for (lbl, position) in &self.meta.label_mappings {
                writeln!(f, "\t{} -> {}", lbl.0, position)?;
            }
            writeln!(f, "}}")?;
        }
        writeln!(f, "{{")?;
        for instr in &self.instructions {
            writeln!(f, "\t{}", instr)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

pub struct ModuleAssociatedBlock {
    pub module: ModuleID,
    pub meta: CodeMeta,
    pub instructions: Vec<Instruction>,
}

struct ModuleBlocks {
    top_level: BlockID,
    blocks: KeyedVec<LocalBlockID, BlockID>,
}

#[derive(Default)]
pub struct CodeBlocks {
    blocks: KeyedVec<BlockID, ModuleAssociatedBlock>,
    modules: KeyedVec<ModuleID, ModuleBlocks>,
}

impl std::ops::Index<BlockID> for CodeBlocks {
    type Output = ModuleAssociatedBlock;

    fn index(&self, index: BlockID) -> &Self::Output {
        &self.blocks[index]
    }
}

impl std::ops::IndexMut<BlockID> for CodeBlocks {
    fn index_mut(&mut self, index: BlockID) -> &mut Self::Output {
        &mut self.blocks[index]
    }
}

impl CodeBlocks {
    fn add_block(&mut self, code_block: CodeBlock, module: ModuleID) -> BlockID {
        self.blocks.push(ModuleAssociatedBlock {
            module,
            instructions: code_block.instructions,
            meta: code_block.meta,
        })
    }

    pub fn add_module(&mut self, module: CompiledModule) -> BlockID {
        let block_count = module.blocks.len() + 1;
        self.blocks.reserve(block_count);
        let mut module_blocks = KeyedVec::with_capacity(block_count);

        let module_id = self.modules.next_key();
        let top_level_block_id = self.add_block(module.top_level, module_id);

        for block in module.blocks.values() {
            let block_id = self.add_block(block, module_id);
            module_blocks.push(block_id);
        }

        self.modules.push(ModuleBlocks {
            top_level: top_level_block_id,
            blocks: module_blocks,
        });

        top_level_block_id
    }

    pub fn add_top_level_block(&mut self, code_block: CodeBlock) -> BlockID {
        let module_id = self.modules.next_key();
        let block_id = self.add_block(code_block, module_id);
        self.modules.push(ModuleBlocks {
            top_level: block_id,
            blocks: keyed_vec![],
        });
        block_id
    }

    pub fn blocks_of_module(&self, module: ModuleID) -> &KeyedVec<LocalBlockID, BlockID> {
        &self.modules[module].blocks
    }
}

pub struct LocalValues {
    pub f: Vec<f64>,
    pub i: Vec<i32>,
    pub s: Vec<Option<String>>,
    pub d: Vec<LuaValue>,
}

impl LocalValues {
    pub fn new(meta: &CodeMeta) -> Self {
        Self {
            f: vec![0.0; meta.local_count.f as usize],
            i: vec![0; meta.local_count.i as usize],
            s: vec![None; meta.local_count.s as usize],
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
    pub value_count: u16,
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
        let mut code_blocks = CodeBlocks::default();
        let dummy_block = CodeBlock {
            meta: Default::default(),
            instructions: vec![/* Assert */],
        };
        let dummy_block_id = code_blocks.add_top_level_block(dummy_block);

        Self {
            accumulators: Accumulators {
                f: 0.0,
                i: 0,
                s: None,
                c: dummy_block_id,
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
                d: [(); ARG_REG_COUNT].map(|_| LuaValue::Nil),
            },
            global_values: GlobalValues::default(),
            code_blocks: CodeBlocks::default(),
            stack: vec![],
        }
    }
}
