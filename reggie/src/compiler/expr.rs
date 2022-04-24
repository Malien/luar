use luar_syn::{BinaryOperator, Expression, UnaryOperator};

use crate::{
    compiler::{compile_fn_call, compile_var_lookup, LocalScopeCompilationState},
    ids::{ArgumentRegisterID, LocalRegisterID},
    ops::Instruction,
};

pub fn compile_expr(expr: &Expression, state: &mut LocalScopeCompilationState) {
    use Instruction::*;

    match expr {
        Expression::Nil => {
            state.push_instr(ConstN);
        }
        Expression::Number(num) if num.is_integer() => {
            state.push_instr(ConstI(num.as_i32()));
            state.push_instr(WrapI);
        }
        Expression::Number(num) => {
            state.push_instr(ConstF(num.as_f64()));
            state.push_instr(WrapF);
        }
        Expression::String(str) => {
            let str_id = state.alloc_string(str.0.clone());
            state.push_instr(ConstS(str_id));
            state.push_instr(WrapS);
        }
        Expression::BinaryOperator { lhs, op, rhs } => {
            compile_binary_op(*op, lhs, rhs, state);
        }
        Expression::Variable(var) => {
            compile_var_lookup(var, state);
        }
        Expression::FunctionCall(fn_call) => {
            compile_fn_call(fn_call, state);
            state.push_instr(LdaProt(ArgumentRegisterID(0)));
        }
        Expression::UnaryOperator {
            op: UnaryOperator::Not,
            exp,
        } => {
            compile_expr(exp, state);
            let true_label = state.alloc_label();
            let cont_label = state.alloc_label();
            state.push_instr(NilTest);
            state.push_instr(JmpEQ(true_label));
            state.push_instr(ConstN);
            state.push_instr(Jmp(cont_label));
            state.push_label(true_label);
            state.push_instr(ConstI(1));
            state.push_instr(WrapI);
            state.push_label(cont_label);
        }
        _ => todo!("Cannot compile \"{}\" expression yet", expr),
    }
}

fn compile_binary_op(
    op: BinaryOperator,
    lhs: &Expression,
    rhs: &Expression,
    state: &mut LocalScopeCompilationState,
) {
    use Instruction::*;

    compile_expr(lhs, state);
    let lhs_reg = state.reg().alloc_dyn();
    state.push_instr(StrLD(lhs_reg));
    compile_expr(rhs, state);
    let rhs_reg = state.reg().alloc_dyn();
    state.push_instr(StrLD(rhs_reg));
    state.push_instr(LdaLD(lhs_reg));

    if let BinaryOperator::Equals = op {
        compile_eq_op(state, rhs_reg, false);
    } else if let BinaryOperator::NotEquals = op {
        compile_eq_op(state, rhs_reg, true);
    } else {
        let instr = match op {
            BinaryOperator::Plus => DAddL,
            BinaryOperator::Minus => DSubL,
            BinaryOperator::Mul => DMulL,
            BinaryOperator::Div => DDivL,
            BinaryOperator::Equals => unreachable!(),
            _ => todo!(),
        };

        state.push_instr(instr(rhs_reg));
    }

    state.reg().free_dyn();
    state.reg().free_dyn();
}

fn compile_eq_op(state: &mut LocalScopeCompilationState, rhs_value: LocalRegisterID, negated: bool) {
    use Instruction::*;
    let true_lbl = state.alloc_label();
    let cont_lbl = state.alloc_label();
    state.push_instr(EqTestLD(rhs_value));
    let jmp_instr = if negated { JmpNE } else { JmpEQ };
    state.push_instr(jmp_instr(true_lbl));
    state.push_instr(ConstN);
    state.push_instr(Jmp(cont_lbl));
    state.push_label(true_lbl);
    state.push_instr(ConstI(1));
    state.push_instr(WrapI);
    state.push_label(cont_lbl);
}
