use luar_syn::Return;

use crate::{ids::ArgumentRegisterID, ops::Instruction};

use super::{compile_expr, LocalFnCompState};

pub fn compile_ret(ret: &Return, state: &mut LocalFnCompState) {
    let Return(expressions) = ret;
    if expressions.len() > 1 {
        todo!("Cannot compile multiple return yet");
    }
    if let Some(expr) = expressions.first() {
        compile_expr(expr, state);
        state.push_instr(Instruction::StrRD(ArgumentRegisterID(0)));
    }
    state.push_instr(Instruction::Ret);
}
