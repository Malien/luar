use std::num::NonZeroU16;

use luar_syn::{Block, Chunk, Expression, FunctionDeclaration, Module, Return, Statement, Conditional, ConditionalTail};

use crate::meta::ReturnCount;

use super::ReturnCountState;

pub fn return_traverse_module(module: &Module) -> ReturnCount {
    let ret = module
        .ret
        .as_ref()
        .map(return_traverse_return)
        .unwrap_or(ReturnCountState::NotSpecified);

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
        .unwrap_or(ReturnCountState::NotSpecified);

    block
        .statements
        .iter()
        .map(return_traverse_statement)
        .fold(ret, ReturnCountState::combine)
}

fn return_traverse_statement(statement: &Statement) -> ReturnCountState {
    match statement {
        Statement::While(while_loop) => return_traverse_block(&while_loop.body),
        Statement::Repeat(repeat_loop) => return_traverse_block(&repeat_loop.body),
        Statement::If(conditional) => return_traverse_conditional(conditional),
        _ => ReturnCountState::NotSpecified,
    }
}

fn return_traverse_conditional(conditional: &Conditional) -> ReturnCountState {
    let body_return_count = return_traverse_block(&conditional.body);

    let tail_return_count = match &conditional.tail {
        ConditionalTail::End => ReturnCountState::NotSpecified,
        ConditionalTail::Else(else_body) => return_traverse_block(else_body),
        ConditionalTail::ElseIf(conditional) => return_traverse_conditional(conditional),
    };

    return ReturnCountState::combine(body_return_count, tail_return_count);
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
