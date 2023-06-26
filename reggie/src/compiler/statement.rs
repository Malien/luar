use luar_syn::{Block, Conditional, ConditionalTail, Statement, WhileLoop};

use crate::ops::Instruction;

use super::{
    compile_assignment, compile_expr, compile_fn_call, compile_local_decl, ret::compile_ret,
    LocalScopeCompilationState,
};

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
        Statement::LocalDeclaration(decl) => {
            compile_local_decl(decl, state);
        }
        Statement::While(while_loop) => {
            compile_while_loop(while_loop, state);
        }
        Statement::Repeat(repeat_loop) => todo!("Compilation of statement \"{repeat_loop}\" is not implemented yet"),
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
        ConditionalTail::Else(ref block) => {
            let else_lbl = state.alloc_label();
            let cont_lbl = state.alloc_label();
            state.push_instr(Instruction::JmpEQ(else_lbl));
            compile_block(&conditional.body, state);
            state.push_instr(Instruction::Jmp(cont_lbl));
            state.push_label(else_lbl);
            compile_block(block, state);
            state.push_label(cont_lbl);
        }
        ConditionalTail::ElseIf(ref elseif) => {
            let else_lbl = state.alloc_label();
            let cont_lbl = state.alloc_label();
            state.push_instr(Instruction::JmpEQ(else_lbl));
            compile_block(&conditional.body, state);
            state.push_instr(Instruction::Jmp(cont_lbl));
            state.push_label(else_lbl);
            compile_conditional(elseif.as_ref(), state);
            state.push_label(cont_lbl);
        }
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

pub fn compile_while_loop(while_loop: &WhileLoop, state: &mut LocalScopeCompilationState) {
    let loop_entry_lbl = state.alloc_label();
    let cont_lbl = state.alloc_label();

    state.push_label(loop_entry_lbl);
    compile_expr(&while_loop.condition, state);
    state.push_instr(Instruction::NilTest);
    state.push_instr(Instruction::JmpEQ(cont_lbl));
    compile_block(&while_loop.body, state);
    state.push_instr(Instruction::Jmp(loop_entry_lbl));
    state.push_label(cont_lbl);
}
