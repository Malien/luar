use std::num::NonZeroU16;

use luar_syn::{Block, Chunk, Expression, FunctionDeclaration, Module, Return, Statement};

use crate::meta::ReturnCount;

use super::ReturnCountState;

pub fn return_traverse_module(module: &Module) -> ReturnCount {
    let ret = module
        .ret
        .as_ref()
        .map(return_traverse_return)
        .unwrap_or_default();

    module
        .chunks
        .iter()
        .filter_map(Chunk::as_statement_ref)
        .map(return_traverse_statement)
        .fold(ret, ReturnCountState::combine)
        .into_return_count()
        .unwrap_or(ReturnCount::Constant(0))
}

pub fn return_traverse_function(function: &FunctionDeclaration) -> ReturnCount {
    return_traverse_block(&function.body)
        .into_return_count()
        .unwrap_or(ReturnCount::Constant(0))
}

fn return_traverse_block(block: &Block) -> ReturnCountState {
    let ret = block
        .ret
        .as_ref()
        .map(return_traverse_return)
        .unwrap_or_default();

    block
        .statements
        .iter()
        .map(return_traverse_statement)
        .fold(ret, ReturnCountState::combine)
}

fn return_traverse_statement(statement: &Statement) -> ReturnCountState {
    let block = match statement {
        Statement::While(while_loop) => &while_loop.body,
        Statement::Repeat(repeat_loop) => &repeat_loop.body,
        Statement::If(conditional) => &conditional.body,
        _ => return ReturnCountState::NotSpecified,
    };
    return_traverse_block(block)
}

fn return_traverse_return(ret: &Return) -> ReturnCountState {
    let count: u16 = ret.0.len().try_into().unwrap();
    match ret.0.last() {
        Some(Expression::FunctionCall(_)) => match NonZeroU16::new(count - 1) {
            Some(count) => ReturnCountState::MinBounded(count),
            None => ReturnCountState::Unbounded,
        },
        Some(_) => ReturnCountState::Constant(count),
        None => ReturnCountState::Constant(0),
    }
}
