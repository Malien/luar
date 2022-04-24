use std::num::NonZeroU16;

use luar_syn::{Expression, Return};

use crate::ids::ArgumentRegisterID;

use super::{compile_expr, compile_fn_call, LocalScopeCompilationState};

pub fn compile_ret(ret: &Return, state: &mut LocalScopeCompilationState) {
    use crate::ops::Instruction::*;
    let Return(expressions) = ret;

    if let Some((last, head)) = expressions.split_last() {
        let head_count = head.len().try_into().unwrap();
        let intermediates = state.reg().alloc_dyn_count(head_count);
        for (expr, local_reg) in head.iter().zip(&intermediates) {
            compile_expr(expr, state);
            state.push_instr(StrLD(local_reg));
        }
        match last {
            Expression::FunctionCall(fn_call) => {
                let tmp = state.reg().alloc_int();
                compile_fn_call(fn_call, state);
                state.push_instr(ConstI(head_count as i32));
                state.push_instr(RDShiftRight);
                state.push_instr(StrLI(tmp));
                state.push_instr(LdaVC);
                state.push_instr(IAddL(tmp));
                state.push_instr(StrVC);
                state.reg().free_int();
                match NonZeroU16::new(head_count) {
                    Some(count) => state.return_count().update_known_min(count),
                    None => state.return_count().update_unknown(),
                }
            }
            expr => {
                compile_expr(expr, state);
                state.push_instr(StrRD(ArgumentRegisterID(head_count)));
                state.push_instr(ConstI(head_count as i32 + 1));
                state.push_instr(StrVC);
                state.return_count().update_known(head_count + 1);
            }
        }
        for (local_reg, idx) in intermediates.into_iter().zip(0..) {
            state.push_instr(LdaLD(local_reg));
            state.push_instr(StrRD(ArgumentRegisterID(idx)));
        }
        state.reg().free_dyn_count(intermediates.count);
    } else {
        state.push_instr(ConstI(0));
        state.push_instr(StrVC);
        state.return_count().update_known(0);
    }

    state.push_instr(Ret);
}
