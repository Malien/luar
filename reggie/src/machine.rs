use enum_map::Enum;
use luar_string::LuaString;

use crate::{
    compiler::CompiledModule,
    global_values::GlobalValues,
    ids::{BlockID, LocalBlockID, ModuleID},
    meta::CodeMeta,
    ops::Instruction,
    LuaValue, TableRef, stdlib::define_stdlib, call_stack::CallStack,
};
use keyed_vec::{keyed_vec, KeyedVec};

// const ARG_REG_COUNT: usize = 16;
// TODO: Implement ExtR in order to make argument register more likely to be in cache(?)
const ARG_REG_COUNT: usize = 32;

// pub const OPTIMIZE: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum DataType {
    Dynamic,
    Int,
    Float,
    String,
    Function,
    NativeFunction,
    Table,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DataType::*;
        match self {
            Dynamic => "D",
            Int => "I",
            Float => "F",
            String => "S",
            Function => "C",
            NativeFunction => "A",
            Table => "T",
        }
        .fmt(f)
    }
}

pub struct ArgumentRegisters {
    pub f: [f64; ARG_REG_COUNT],
    pub i: [i32; ARG_REG_COUNT],
    pub s: [LuaString; ARG_REG_COUNT],
    pub t: [Option<TableRef>; ARG_REG_COUNT],
    pub d: [LuaValue; ARG_REG_COUNT],
}

pub struct Accumulators {
    pub f: f64,
    pub i: i32,
    pub s: LuaString,
    pub c: BlockID,
    pub t: Option<TableRef>,
    pub d: LuaValue,
}

impl TestFlag {
    pub fn from_bool(v: bool) -> Self {
        match v {
            true => Self::EQ,
            false => Self::NE,
        }
    }

    pub fn test_succeeded(self) -> bool {
        matches!(self, Self::EQ | Self::LT)
    }

    pub fn test_failed(self) -> bool {
        matches!(self, Self::NE | Self::GT)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestFlag {
    EQ = 0b00,
    NE = 0b01,
    LT = 0b10,
    GT = 0b11,
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
        let has_any_locals = lc.values().any(|v| *v != 0);
        if has_any_locals {
            writeln!(f, "locals {{")?;
            for (reg_type, count) in lc {
                if *count != 0 {
                    writeln!(f, "\t{}: {}", reg_type, count)?;
                }
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

#[derive(Debug)]
pub struct ModuleAssociatedBlock {
    pub module: ModuleID,
    pub meta: CodeMeta,
    pub instructions: Vec<Instruction>,
}

struct ModuleBlocks {
    // top_level: BlockID,
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

        for block in module.blocks.into_values() {
            let block_id = self.add_block(block, module_id);
            module_blocks.push(block_id);
        }

        self.modules.push(ModuleBlocks {
            // top_level: top_level_block_id,
            blocks: module_blocks,
        });

        top_level_block_id
    }

    pub fn add_top_level_block(&mut self, code_block: CodeBlock) -> BlockID {
        let module_id = self.modules.next_key();
        let block_id = self.add_block(code_block, module_id);
        self.modules.push(ModuleBlocks {
            // top_level: block_id,
            blocks: keyed_vec![],
        });
        block_id
    }

    pub fn blocks_of_module(&self, module: ModuleID) -> &KeyedVec<LocalBlockID, BlockID> {
        &self.modules[module].blocks
    }
}

pub struct Machine {
    pub accumulators: Accumulators,
    pub program_counter: ProgramCounter,
    pub value_count: u16,
    pub test_flag: TestFlag,
    pub type_test_result: TypeTestResult,
    pub argument_registers: ArgumentRegisters,
    pub global_values: GlobalValues,
    pub code_blocks: CodeBlocks,
    pub stack: CallStack,
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
                s: LuaString::default(),
                c: dummy_block_id,
                t: None,
                d: LuaValue::Nil,
            },
            program_counter: ProgramCounter {
                block: BlockID(0),
                position: 0,
            },
            value_count: 0,
            test_flag: TestFlag::EQ,
            type_test_result: TypeTestResult::Nil,
            argument_registers: ArgumentRegisters {
                f: [0.0; ARG_REG_COUNT],
                i: [0; ARG_REG_COUNT],
                s: [(); ARG_REG_COUNT].map(|_| LuaString::default()),
                t: [(); ARG_REG_COUNT].map(|_| None),
                d: [(); ARG_REG_COUNT].map(|_| LuaValue::Nil),
            },
            global_values: GlobalValues::default(),
            code_blocks: CodeBlocks::default(),
            stack: CallStack::default(),
        }
    }

    pub fn with_stdlib() -> Self {
        let mut machine = Self::new();
        define_stdlib(&mut machine.global_values);
        machine
    }
}
