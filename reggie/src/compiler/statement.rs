use luar_syn::{Assignment, Block, Conditional, ConditionalTail, Statement, Var};

use crate::ops::Instruction;

use super::{compile_expr, ret::compile_ret, LocalFnCompState};

pub fn compile_statement(statement: &Statement, state: &mut LocalFnCompState) {
    match statement {
        Statement::If(conditional) => {
            compile_conditional(conditional, state);
        }
        Statement::Assignment(assignment) => {
            compile_assignment(assignment, state);
        }
        _ => todo!("Compiling statement \"{}\" is not implemented", statement),
    };
}

pub fn compile_conditional(conditional: &Conditional, state: &mut LocalFnCompState) {
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

pub fn compile_block(block: &Block, state: &mut LocalFnCompState) {
    let mut inner_scope = state.inner_scope();
    for statement in &block.statements {
        compile_statement(statement, &mut inner_scope);
    }
    if let Some(ret) = &block.ret {
        compile_ret(ret, state);
    }
}

pub fn compile_assignment(assignment: &Assignment, state: &mut LocalFnCompState) {
    let intermediates = state
        .reg()
        .alloc_dyn_count(assignment.names.len().try_into().unwrap());
    for i in 0..intermediates.count {
        if let Some(expr) = assignment.values.get(i as usize) {
            compile_expr(expr, state);
        } else {
            state.push_instr(Instruction::ConstN);
        }
        state.push_instr(Instruction::StrLD(intermediates.at(i)));
    }
    if assignment.values.len() > assignment.names.len() {
        for expr in &assignment.values[(intermediates.count as usize)..] {
            compile_expr(expr, state);
        }
    }
    for (var, i) in assignment.names.iter().zip(0..) {
        state.push_instr(Instruction::LdaLD(intermediates.at(i)));
        compile_store(var, state);
    }
    state.reg().free_dyn_count(intermediates.count);
}

pub fn compile_store(var: &Var, state: &mut LocalFnCompState) {
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
