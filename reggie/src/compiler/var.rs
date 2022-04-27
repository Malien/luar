use luar_lex::Ident;
use luar_syn::Var;

use super::{LocalScopeCompilationState, VarLookup};

pub fn compile_var_lookup(var: &Var, state: &mut LocalScopeCompilationState) {
    match var {
        Var::Named(ident) => compile_named_lookup(ident, state),
        Var::PropertyAccess { from, property } => compile_property_lookup(from, property, state),
        _ => todo!("Cannot compile var lookup into tables \"{}\", yet", var),
    }
}

fn compile_property_lookup(
    from: &Box<Var>,
    property: &Ident,
    state: &mut LocalScopeCompilationState,
) {
    use crate::ops::Instruction::*;

    compile_var_lookup(from.as_ref(), state);
    let property_id = state.alloc_string(property.as_ref().into());
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
