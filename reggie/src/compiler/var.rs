use luar_syn::Var;

use super::{LocalScopeCompilationState, VarLookup};

pub fn compile_var_lookup(var: &Var, state: &mut LocalScopeCompilationState) {
    use crate::ops::Instruction::*;

    match var {
        Var::Named(ident) => {
            match state.lookup_var(ident.as_ref()) {
                VarLookup::Argument(reg) => state.push_instr(LdaRD(reg)),
                VarLookup::Local(reg) => state.push_instr(LdaLD(reg)),
                VarLookup::GlobalCell(cell) => state.push_instr(LdaDGl(cell)),
            };
        },
        _ => todo!("Cannot compile var lookup into tables \"{}\", yet", var)
    }
}
