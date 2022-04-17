use luar_syn::{Statement, Conditional, ConditionalTail, Block};

use crate::ops::Instruction;

use super::{LocalFnCompState, compile_expr, ret::compile_ret};

pub fn compile_statement(statement: &Statement, state: &mut LocalFnCompState) {
    match statement {
        Statement::If(conditional) => {
            compile_conditional(conditional, state);
        },
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
        _ => todo!("Cannot compile else and elseif cases yet")
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