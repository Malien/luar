use std::collections::HashMap;

use crate::lang::{LuaFunction, LuaValue, TableRef};

use super::ids::GlobalCellID;

const ARG_REG_COUNT: usize = 16;

pub struct ArgumentRegisters {
    pub f: [f64; ARG_REG_COUNT],
    pub i: [i32; ARG_REG_COUNT],
    pub s: [String; ARG_REG_COUNT],
    pub t: [TableRef; ARG_REG_COUNT],
    pub c: [LuaFunction; ARG_REG_COUNT],
    pub d: [LuaValue; ARG_REG_COUNT],
}

pub struct Accumulators {
    pub f: f64,
    pub i: i32,
    pub s: String,
    pub t: TableRef,
    pub c: LuaFunction,
    pub d: LuaValue,
}

pub enum EqualityFlag {
    EQ,
    NE,
}

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

impl Default for GlobalValueCell {
    fn default() -> Self {
        Self { value: LuaValue::Nil }
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

impl GlobalValues {
    pub fn cell_for_name(&mut self, ident: impl Into<String>) -> GlobalCellID {
        *self.mapping.entry(ident.into()).or_insert_with(|| {
            let idx = self.cells.len();
            self.cells.push(GlobalValueCell::default());
            GlobalCellID(idx.try_into().unwrap())
        })
    }
}

pub struct Machine {
    pub accumulators: Accumulators,
    pub program_counter: usize,
    pub value_count: u32,
    pub equality_flag: EqualityFlag,
    pub ordering_flag: OrderingFlag,
    pub type_test_result: TypeTestResult,
    pub argument_registers: ArgumentRegisters,
    pub global_values: GlobalValues,
}
