use luar_syn::{
    Assignment, Block, Conditional, ConditionalTail, Declaration, Expression, Statement, Var,
};

use crate::{ids::ArgumentRegisterID, ops::Instruction};

use super::{
    compile_expr, compile_fn_call, ret::compile_ret, LocalRegisterSpan, LocalScopeCompilationState,
    RegisterAllocator,
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
    let (head, last) = assignment.values.split_last();
    let (head_regs, tail_regs) =
        alloc_assignment_locals(state.reg(), assignment.names.len().get(), head.len());

    compile_assignment_head(head, head_regs, state);
    compile_assignment_tail(last, tail_regs, state);

    for (var, local_reg) in assignment
        .names
        .iter()
        .zip(head_regs.into_iter().chain(&tail_regs))
    {
        state.push_instr(Instruction::LdaLD(local_reg));
        compile_store(var, state);
    }

    state.reg().free_dyn_count(head_regs.count);
    state.reg().free_dyn_count(tail_regs.count);
}

fn alloc_assignment_locals(
    reg_alloc: &mut RegisterAllocator,
    total_count: usize,
    head_count: usize,
) -> (LocalRegisterSpan, LocalRegisterSpan) {
    let head_reg_count = head_count.try_into().unwrap();
    let head_regs = reg_alloc.alloc_dyn_count(head_reg_count);
    let tail_reg_count: u16 = total_count
        .checked_sub(head_count)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .unwrap_or(0);
    let tail_regs = reg_alloc.alloc_dyn_count(tail_reg_count);
    (head_regs, tail_regs)
}

fn compile_assignment_tail(
    last: &Expression,
    left_unassigned: LocalRegisterSpan,
    state: &mut LocalScopeCompilationState,
) {
    match last {
        Expression::FunctionCall(fn_call) if left_unassigned.count > 1 => {
            compile_fn_call(fn_call, state);

            for (local_reg, idx) in left_unassigned.into_iter().zip(0..) {
                state.push_instr(Instruction::LdaProt(ArgumentRegisterID(idx)));
                state.push_instr(Instruction::StrLD(local_reg));
            }
        }
        expr => {
            compile_expr(expr, state);
            if let Some(unassigned_reg) = left_unassigned.try_at(0) {
                state.push_instr(Instruction::StrLD(unassigned_reg))
            }
        }
    }
}

fn compile_assignment_head(
    head: &[Expression],
    intermediates: LocalRegisterSpan,
    state: &mut LocalScopeCompilationState,
) {
    let mut locals_iter = intermediates.into_iter();
    for expr in head {
        compile_expr(expr, state);
        if let Some(local_reg) = locals_iter.next() {
            state.push_instr(Instruction::StrLD(local_reg));
        }
    }
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

pub fn compile_local_decl(decl: &Declaration, state: &mut LocalScopeCompilationState) {
    if let Some((last, head)) = decl.initial_values.split_last() {
        let (head_regs, tail_regs) =
            alloc_assignment_locals(state.reg(), decl.names.len().get(), head.len());

        compile_assignment_head(head, head_regs, state);
        compile_assignment_tail(last, tail_regs, state);

        for (ident, local_reg) in decl
            .names
            .iter()
            .zip(head_regs.into_iter().chain(&tail_regs))
        {
            state.define_local(ident.to_string(), local_reg);
        }
    } else {
        let locals_count = decl.names.len().try_into().unwrap();
        let locals = state.reg().alloc_dyn_nonzero(locals_count);
        for (ident, local_reg) in decl.names.iter().zip(&locals) {
            state.define_local(ident.to_string(), local_reg);
        }
    };
}
