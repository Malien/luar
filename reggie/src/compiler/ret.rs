use luar_syn::{Expression, Return};

use crate::{ids::ArgumentRegisterID, machine::DataType, ops::Instruction};

use super::{compile_expr, compile_fn_call, LocalScopeCompilationState, LocalRegisterSpan};

pub fn compile_ret(Return(expressions): &Return, state: &mut LocalScopeCompilationState) {
    if let Some((last, head)) = expressions.split_last() {
        compile_nonempty_return(head, last, state);
    } else if state.return_count().is_varying() {
        state.push_instr(Instruction::ConstI(0));
        state.push_instr(Instruction::StrVC);
    }

    state.push_instr(Instruction::Ret);
}

fn compile_nonempty_return(
    head: &[Expression],
    last: &Expression,
    state: &mut LocalScopeCompilationState,
) {
    use crate::ops::Instruction::*;

    let head_count = head.len().try_into().unwrap();
    let intermediates = state.reg().alloc_count(DataType::Dynamic, head_count);

    for (expr, local_reg) in head.iter().zip(&intermediates) {
        compile_expr(expr, state);
        state.push_instr(StrLD(local_reg));
    }
    compile_tail_return(last, head_count, state);
    move_intermediates_into_arguments(intermediates, state);

    state
        .reg()
        .free_count(DataType::Dynamic, intermediates.count);
}

fn move_intermediates_into_arguments(intermediates: LocalRegisterSpan, state: &mut LocalScopeCompilationState) {
    for (local_reg, idx) in intermediates.into_iter().zip(0..) {
        state.push_instr(Instruction::LdaLD(local_reg));
        state.push_instr(Instruction::StrRD(ArgumentRegisterID(idx)));
    }
}

fn compile_tail_return(last: &Expression, head_count: u16, state: &mut LocalScopeCompilationState) {
    match last {
        Expression::FunctionCall(fn_call) => {
            compile_last_function_call_in_return(fn_call, head_count, state);
        }
        expr => {
            compile_expr(expr, state);
            state.push_instr(Instruction::StrRD(ArgumentRegisterID(head_count)));
            if state.return_count().is_varying() {
                state.push_instr(Instruction::ConstI(head_count as i32 + 1));
                state.push_instr(Instruction::StrVC);
            }
        }
    }
}

fn compile_last_function_call_in_return(
    fn_call: &luar_syn::FunctionCall,
    head_count: u16,
    state: &mut LocalScopeCompilationState,
) {
    use crate::ops::Instruction::*;

    let tmp = state.reg().alloc(DataType::Int);
    compile_fn_call(fn_call, state);
    state.push_instr(ConstI(head_count as i32));
    state.push_instr(RDShiftRight);
    state.push_instr(StrLI(tmp));
    state.push_instr(LdaVC);
    state.push_instr(IAddL(tmp));
    state.push_instr(StrVC);
    state.reg().free(DataType::Int);
}
