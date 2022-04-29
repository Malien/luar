use luar_syn::{BinaryOperator, Expression, TableConstructor, UnaryOperator};

use crate::{
    compiler::{compile_fn_call, compile_var_lookup, LocalScopeCompilationState},
    ids::{ArgumentRegisterID, JmpLabel, LocalRegisterID},
    machine::DataType,
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
            compile_not(state);
        }
        Expression::UnaryOperator {
            op: UnaryOperator::Minus,
            exp,
        } => {
            compile_expr(exp, state);
            state.push_instr(NegD);
        }
        Expression::TableConstructor(table) => {
            compile_table_constructor(table, state);
        }
    }
}

fn compile_not(state: &mut LocalScopeCompilationState) {
    use Instruction::*;

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

fn compile_binary_op(
    op: BinaryOperator,
    lhs: &Expression,
    rhs: &Expression,
    state: &mut LocalScopeCompilationState,
) {
    compile_expr(lhs, state);

    match categorize_op(op) {
        BinaryOpType::ShortCircuits(op) => compile_short_circuit_op(op, rhs, state),
        BinaryOpType::Regular(op) => compile_regular_binary_op(op, rhs, state),
    }

}

fn compile_comparison(
    rhs_reg: LocalRegisterID,
    jmp_instr: impl FnOnce(JmpLabel) -> Instruction,
    state: &mut LocalScopeCompilationState,
) {
    use Instruction::*;

    let true_lbl = state.alloc_label();
    let cont_lbl = state.alloc_label();
    state.push_instr(TestLD(rhs_reg));
    state.push_instr(jmp_instr(true_lbl));
    state.push_instr(ConstN);
    state.push_instr(Jmp(cont_lbl));
    state.push_label(true_lbl);
    state.push_instr(ConstI(1));
    state.push_instr(WrapI);
    state.push_label(cont_lbl);
}

enum ShortCircuitOp {
    And,
    Or,
}

enum RegularOp {
    Equals,
    NotEquals,
    Plus,
    Minus,
    Mul,
    Div,
    Less,
    Greater,
    LessOrEquals,
    GreaterOrEquals,
    Concat,
    Exp,
}

enum BinaryOpType {
    ShortCircuits(ShortCircuitOp),
    Regular(RegularOp),
}

fn categorize_op(binary_op: BinaryOperator) -> BinaryOpType {
    use BinaryOpType::*;

    match binary_op {
        BinaryOperator::And => ShortCircuits(ShortCircuitOp::And),
        BinaryOperator::Or => ShortCircuits(ShortCircuitOp::Or),

        BinaryOperator::Less => Regular(RegularOp::Less),
        BinaryOperator::Greater => Regular(RegularOp::Greater),
        BinaryOperator::LessOrEquals => Regular(RegularOp::LessOrEquals),
        BinaryOperator::GreaterOrEquals => Regular(RegularOp::GreaterOrEquals),
        BinaryOperator::NotEquals => Regular(RegularOp::NotEquals),
        BinaryOperator::Equals => Regular(RegularOp::Equals),
        BinaryOperator::Concat => Regular(RegularOp::Concat),
        BinaryOperator::Plus => Regular(RegularOp::Plus),
        BinaryOperator::Minus => Regular(RegularOp::Minus),
        BinaryOperator::Mul => Regular(RegularOp::Mul),
        BinaryOperator::Div => Regular(RegularOp::Div),
        BinaryOperator::Exp => Regular(RegularOp::Exp),
    }
}

fn compile_short_circuit_op(
    op: ShortCircuitOp,
    rhs: &Expression,
    state: &mut LocalScopeCompilationState,
) {
    use Instruction::*;

    let jmp_instr = match op {
        ShortCircuitOp::And => JmpEQ,
        ShortCircuitOp::Or => JmpNE,
    };

    let short_circuit_lbl = state.alloc_label();
    state.push_instr(NilTest);
    state.push_instr(jmp_instr(short_circuit_lbl));
    compile_expr(rhs, state);
    state.push_label(short_circuit_lbl);
}

fn compile_regular_binary_op(op: RegularOp, rhs: &Expression, state: &mut LocalScopeCompilationState) {
    use Instruction::*;

    let lhs_reg = state.reg().alloc(DataType::Dynamic);
    state.push_instr(StrLD(lhs_reg));
    compile_expr(rhs, state);
    let rhs_reg = state.reg().alloc(DataType::Dynamic);
    state.push_instr(StrLD(rhs_reg));
    state.push_instr(LdaLD(lhs_reg));

    match op {
        RegularOp::Equals => compile_eq_op(state, rhs_reg, false),
        RegularOp::NotEquals => compile_eq_op(state, rhs_reg, true),
        RegularOp::Plus => state.push_instr(DAddL(rhs_reg)),
        RegularOp::Minus => state.push_instr(DSubL(rhs_reg)),
        RegularOp::Mul => state.push_instr(DMulL(rhs_reg)),
        RegularOp::Div => state.push_instr(DDivL(rhs_reg)),
        RegularOp::Less => compile_comparison(rhs_reg, JmpLT, state),
        RegularOp::Greater => compile_comparison(rhs_reg, JmpGT, state),
        RegularOp::LessOrEquals => compile_comparison(rhs_reg, JmpLE, state),
        RegularOp::GreaterOrEquals => compile_comparison(rhs_reg, JmpGE, state),

        RegularOp::Concat => {
            todo!("Cannot compile the use of concatenation operator '..' yet")
        }
        RegularOp::Exp => todo!("Cannot compile the use of exponentiation operator '^' yet"),
    }

    state.reg().free(DataType::Dynamic);
    state.reg().free(DataType::Dynamic);
}

fn compile_eq_op(
    state: &mut LocalScopeCompilationState,
    rhs_value: LocalRegisterID,
    negated: bool,
) {
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

pub fn compile_table_constructor(table: &TableConstructor, state: &mut LocalScopeCompilationState) {
    use Instruction::*;

    let table_reg = state.reg().alloc(DataType::Table);

    state.push_instr(NewT);
    state.push_instr(StrLT(table_reg));

    for value in &table.lfield {
        compile_expr(value, state);
        state.push_instr(LdaLT(table_reg));
        state.push_instr(PushD);
    }

    for (ident, value) in &table.ffield {
        let ident_id = state.alloc_string(ident.as_ref().to_owned());
        compile_expr(value, state);
        state.push_instr(ConstS(ident_id));
        state.push_instr(LdaLT(table_reg));
        state.push_instr(AssocASD);
    }

    state.push_instr(WrapT);

    state.reg().free(DataType::Table);
}
