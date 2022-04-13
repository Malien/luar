use crate::{
    reggie::{
        compiler::{LocalFnCompState, VarLookup},
        ids::LocalRegisterID,
        ops::Instruction,
    },
    syn,
};

pub fn compile_expr(expr: &syn::Expression, state: &mut LocalFnCompState) {
    use Instruction::*;

    match expr {
        syn::Expression::Nil => {
            state.push_instr(ConstN);
        }
        syn::Expression::Number(num) if num.is_integer() => {
            state.push_instr(ConstI(num.as_i32()));
            state.push_instr(WrapI);
        }
        syn::Expression::Number(num) => {
            state.push_instr(ConstF(num.as_f64()));
            state.push_instr(WrapF);
        }
        syn::Expression::String(str) => {
            let str_id = state.alloc_string(str.0.clone());
            state.push_instr(ConstS(str_id));
            state.push_instr(WrapS);
        }
        syn::Expression::BinaryOperator { lhs, op, rhs } => {
            compile_binary_op(*op, lhs, rhs, state);
        }
        syn::Expression::Variable(syn::Var::Named(ident)) => {
            match state.lookup_var(ident.as_ref()) {
                VarLookup::Argument(reg) => state.push_instr(LdaRD(reg)),
                VarLookup::Local(reg) => state.push_instr(LdaLD(reg)),
                VarLookup::GlobalCell(cell) => state.push_instr(LdaDGl(cell)),
            };
        }
        _ => todo!(),
    }
}

fn compile_binary_op(
    op: syn::BinaryOperator,
    lhs: &syn::Expression,
    rhs: &syn::Expression,
    state: &mut LocalFnCompState,
) {
    use Instruction::*;

    compile_expr(lhs, state);
    let reg = state.reg().alloc_dyn();
    state.push_instr(StrLD(reg));
    compile_expr(rhs, state);

    if let syn::BinaryOperator::Equals = op {
        compile_eq_op(state, reg);
    } else {
        let instr = match op {
            syn::BinaryOperator::Plus => DAddL,
            syn::BinaryOperator::Minus => DSubL,
            syn::BinaryOperator::Mul => DMulL,
            syn::BinaryOperator::Div => DDivL,
            syn::BinaryOperator::Equals => unreachable!(),
            _ => todo!(),
        };

        state.push_instr(instr(reg));
    }

    state.reg().free_dyn();
}

fn compile_eq_op(state: &mut LocalFnCompState, lhs_value: LocalRegisterID) {
    use Instruction::*;
    let true_lbl = state.alloc_label();
    let cont_lbl = state.alloc_label();
    state.push_instr(EqTestLD(lhs_value));
    state.push_instr(JmpEQ(true_lbl));
    state.push_instr(ConstN);
    state.push_instr(Jmp(cont_lbl));
    state.push_label(true_lbl);
    state.push_instr(ConstI(1));
    state.push_instr(WrapI);
    state.push_label(cont_lbl);
}
