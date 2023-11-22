use luar_syn::{Assignment, Declaration, Expression, Var};

use crate::{ids::ArgumentRegisterID, machine::DataType, ops::Instruction};

use super::{
    compile_expr, compile_fn_call, compile_var_lookup, LocalRegisterSpan,
    LocalScopeCompilationState, RegisterAllocator,
};

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

    state.reg().free_count(DataType::Dynamic, head_regs.count);
    state.reg().free_count(DataType::Dynamic, tail_regs.count);
}

fn alloc_assignment_locals(
    reg_alloc: &mut RegisterAllocator,
    total_count: usize,
    head_count: usize,
) -> (LocalRegisterSpan, LocalRegisterSpan) {
    let head_reg_count = head_count.try_into().unwrap();
    let head_regs = reg_alloc.alloc_count(DataType::Dynamic, head_reg_count);
    let tail_reg_count: u16 = total_count
        .checked_sub(head_count)
        .map(TryInto::try_into)
        .map(Result::unwrap)
        .unwrap_or(0);
    let tail_regs = reg_alloc.alloc_count(DataType::Dynamic, tail_reg_count);
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
    use Instruction::*;

    match var {
        Var::Named(ident) => {
            let store_instruction = match state.lookup_var(ident) {
                super::VarLookup::Argument(reg) => StrRD(reg),
                super::VarLookup::Local(reg) => StrLD(reg),
                super::VarLookup::GlobalCell(cell) => StrDGl(cell),
            };
            state.push_instr(store_instruction);
        }
        Var::PropertyAccess { from, property } => {
            let reg = state.reg().alloc(DataType::Dynamic);
            let property_id = state.alloc_string(property.as_ref());
            let is_table_lbl = state.alloc_label();

            state.push_instr(StrLD(reg));
            compile_var_lookup(from, state);
            state.push_instr(CastT);
            state.push_instr(ConstS(property_id));
            state.push_instr(JmpEQ(is_table_lbl));
            state.push_instr(TablePropertyAssignError);
            state.push_label(is_table_lbl);
            state.push_instr(LdaLD(reg));
            state.push_instr(AssocASD);

            state.reg().free(DataType::Dynamic);
        }
        Var::MemberLookup { from, value } => {
            let value_reg = state.reg().alloc(DataType::Dynamic);
            let is_table_lbl = state.alloc_label();

            let table_reg = state.reg().alloc(DataType::Dynamic);

            state.push_instr(StrLD(value_reg));
            compile_var_lookup(from, state);
            state.push_instr(StrLD(table_reg));

            let key_reg = state.reg().alloc(DataType::Dynamic);
            compile_expr(value, state);
            state.push_instr(StrLD(key_reg));

            state.push_instr(LdaLD(table_reg));
            state.push_instr(CastT);
            state.push_instr(JmpEQ(is_table_lbl));
            state.push_instr(TableMemberAssignErrorL(key_reg));
            state.push_label(is_table_lbl);
            state.push_instr(LdaLD(key_reg));
            state.push_instr(AssocLD(value_reg));

            state.reg().free(DataType::Dynamic);
            state.reg().free(DataType::Dynamic);
            state.reg().free(DataType::Dynamic);
        }
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
        let locals = state.reg().alloc_nonzero(DataType::Dynamic, locals_count);
        for (ident, local_reg) in decl.names.iter().zip(&locals) {
            state.define_local(ident.to_string(), local_reg);
        }
    };
}
