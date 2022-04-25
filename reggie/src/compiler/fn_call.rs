use luar_syn::FunctionCall;

use crate::{ids::ArgumentRegisterID, machine::DataType, ops::Instruction};

use super::{compile_expr, compile_var_lookup, LocalScopeCompilationState};

pub fn compile_fn_call(call: &FunctionCall, state: &mut LocalScopeCompilationState) {
    use Instruction::*;

    match call {
        FunctionCall::Function { func, args } => match args {
            luar_syn::FunctionCallArgs::Arglist(args) => {
                let locals = state
                    .reg()
                    .alloc_count(DataType::Dynamic, args.len().try_into().unwrap());
                for (expr, idx) in args.iter().zip(0..) {
                    compile_expr(expr, state);
                    state.push_instr(StrLD(locals.at(idx)));
                }
                for (local, idx) in locals.into_iter().zip(0..) {
                    state.push_instr(LdaLD(local));
                    state.push_instr(StrRD(ArgumentRegisterID(idx)));
                }
                state.push_instr(ConstI(locals.count as i32));
                state.push_instr(StrVC);
                compile_var_lookup(func, state);
                state.push_instr(DCall);
                state.reg().free_count(DataType::Dynamic, locals.count);
            }
            luar_syn::FunctionCallArgs::Table(table) => todo!(
                "Cannot compile function calls with tables \"{} {}\" as arguments yet",
                func,
                table
            ),
        },
        FunctionCall::Method { func, method, args } => {
            todo!("Cannot compile method call {}:{}{} yet", func, method, args)
        }
    }
}
