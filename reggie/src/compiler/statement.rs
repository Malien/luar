use std::num::NonZeroU16;

use luar_syn::{Assignment, Block, Conditional, ConditionalTail, Expression, Statement, Var};

use crate::{ids::ArgumentRegisterID, ops::Instruction};

use super::{compile_expr, compile_fn_call, ret::compile_ret, LocalScopeCompilationState};

pub fn compile_statement(statement: &Statement, state: &mut LocalScopeCompilationState) {
    match statement {
        Statement::If(conditional) => {
            compile_conditional(conditional, state);
        }
        Statement::Assignment(assignment) => {
            compile_assignment(assignment, state);
        }
        Statement::FunctionCall(fn_call) => {
            compile_fn_call(fn_call, state);
        }
        _ => todo!("Compiling statement \"{}\" is not implemented", statement),
    };
}

pub fn compile_conditional(conditional: &Conditional, state: &mut LocalScopeCompilationState) {
    compile_expr(&conditional.condition, state);
    state.push_instr(Instruction::NilTest);

    match conditional.tail {
        ConditionalTail::End => {
            let cont_lbl = state.alloc_label();
            state.push_instr(Instruction::JmpEQ(cont_lbl));
            compile_block(&conditional.body, state);
            state.push_label(cont_lbl);
        }
        _ => todo!("Cannot compile else and elseif clauses yet"),
    };
}

pub fn compile_block(block: &Block, state: &mut LocalScopeCompilationState) {
    let mut inner_scope = state.inner_scope();
    for statement in &block.statements {
        compile_statement(statement, &mut inner_scope);
    }
    if let Some(ret) = &block.ret {
        compile_ret(ret, state);
    }
}

pub fn compile_assignment(assignment: &Assignment, state: &mut LocalScopeCompilationState) {
    let count: NonZeroU16 = assignment.names.len().try_into().unwrap();
    let intermediates = state.reg().alloc_dyn_nonzero(count);

    let (head, last) = assignment.values.split_last();
    for (expr, idx) in head.iter().zip(0..) {
        compile_expr(expr, state);
        if let Some(local_reg) = intermediates.try_at(idx) {
            state.push_instr(Instruction::StrLD(local_reg));
        }
    }

    match last {
        Expression::FunctionCall(fn_call) if assignment.names.len().get() > head.len() + 1 => {
            compile_fn_call(fn_call, state);
            let filled_in_names_count: u16 = head.len().try_into().unwrap();
            let left_unfilled_names = count.get() - filled_in_names_count;

            for idx in 0..left_unfilled_names {
                state.push_instr(Instruction::LdaProt(ArgumentRegisterID(idx)));
                state.push_instr(Instruction::StrLD(
                    intermediates.at(filled_in_names_count + idx),
                ));
            }
        }
        expr => {
            compile_expr(expr, state);
            state.push_instr(Instruction::StrLD(intermediates.last()))
        }
    }

    for (var, i) in assignment.names.iter().zip(0..) {
        state.push_instr(Instruction::LdaLD(intermediates.at(i)));
        compile_store(var, state);
    }
    state.reg().free_dyn_count(intermediates.count.get());
}

pub fn compile_store(var: &Var, state: &mut LocalScopeCompilationState) {
    match var {
        Var::Named(ident) => {
            let store_instruction = match state.lookup_var(ident) {
                super::VarLookup::Argument(reg) => Instruction::StrRD(reg),
                super::VarLookup::Local(reg) => Instruction::StrLD(reg),
                super::VarLookup::GlobalCell(cell) => Instruction::StrDGl(cell),
            };
            state.push_instr(store_instruction);
        }
        _ => todo!("Cannot compile stores to {} yet", var),
    }
}
