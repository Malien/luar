use crate::{
    reggie::{
        compiler::{LocalFnCompState, VarLookup},
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

    let instr = match op {
        syn::BinaryOperator::Plus => DAddL,
        syn::BinaryOperator::Minus => DSubL,
        syn::BinaryOperator::Mul => DMulL,
        syn::BinaryOperator::Div => DDivL,
        _ => todo!(),
    };

    state.push_instr(instr(reg));
    state.reg().free_dyn();
}
