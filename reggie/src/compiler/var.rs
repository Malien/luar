use luar_lex::Ident;
use luar_syn::{Var, Expression};

use crate::{compiler::compile_expr, machine::DataType};

use super::{LocalScopeCompilationState, VarLookup};

pub fn compile_var_lookup(var: &Var, state: &mut LocalScopeCompilationState) {
    match var {
        Var::Named(ident) => compile_named_lookup(ident, state),
        Var::PropertyAccess { from, property } => {
            compile_property_lookup(from, property, state)
        }
        Var::MemberLookup { from, value } => {
            compile_member_lookup(from, value, state)
        }
    }
}

fn compile_member_lookup(from: &Var, value: &Expression, state: &mut LocalScopeCompilationState) {
    use crate::ops::Instruction::*;

    let tmp_reg = state.reg().alloc(DataType::Dynamic);
    compile_expr(value, state);
    state.push_instr(StrLD(tmp_reg));

    let is_table_lbl = state.alloc_label();
    compile_var_lookup(from, state);
    state.push_instr(CastT);
    state.push_instr(JmpEQ(is_table_lbl));
    state.push_instr(TableMemberLookupErrorL(tmp_reg));
    state.push_label(is_table_lbl);
    state.push_instr(LdaLD(tmp_reg));
    state.push_instr(LdaAssocAD);

    state.reg().free(DataType::Dynamic);
}

fn compile_property_lookup(from: &Var, property: &Ident, state: &mut LocalScopeCompilationState) {
    use crate::ops::Instruction::*;

    compile_var_lookup(from, state);
    let property_id = state.alloc_string(property.as_ref());
    let is_table_lbl = state.alloc_label();
    state.push_instr(ConstS(property_id));
    state.push_instr(CastT);
    state.push_instr(JmpEQ(is_table_lbl));
    state.push_instr(TablePropertyLookupError);
    state.push_label(is_table_lbl);
    state.push_instr(LdaAssocAS);
}

fn compile_named_lookup(ident: &Ident, state: &mut LocalScopeCompilationState) {
    use crate::ops::Instruction::*;

    match state.lookup_var(ident.as_ref()) {
        VarLookup::Argument(reg) => state.push_instr(LdaRD(reg)),
        VarLookup::Local(reg) => state.push_instr(LdaLD(reg)),
        VarLookup::GlobalCell(cell) => state.push_instr(LdaDGl(cell)),
    };
}
