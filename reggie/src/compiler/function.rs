use luar_lex::Ident;
use luar_syn::{FunctionDeclaration, Return, Var};

use crate::{
    compiler::{
        compile_statement, ret::compile_ret, FunctionCompilationState, LocalScopeCompilationState,
    },
    ids::{ArgumentRegisterID, LocalBlockID},
    machine::{CodeBlock, DataType},
    meta::{ArgumentCount, CodeMeta, ReturnCount, FunctionKind},
    ops::Instruction,
    GlobalValues,
};

use super::return_traversal::return_traverse_function;

pub fn compile_function(decl: &FunctionDeclaration, global_values: &mut GlobalValues) -> CodeBlock {
    let return_count = return_traverse_function(decl);
    let mut state =
        FunctionCompilationState::with_args(decl.args.iter().cloned(), global_values, return_count);
    let mut root_scope = LocalScopeCompilationState::new(&mut state);

    alias_arguments(&decl.args, &mut root_scope);

    for statement in &decl.body.statements {
        compile_statement(statement, &mut root_scope);
    }

    let empty_ret = Return(vec![]);
    let ret = decl.body.ret.as_ref().unwrap_or(&empty_ret);
    compile_ret(ret, &mut root_scope);

    let debug_name = match decl.name {
        luar_syn::FunctionName::Plain(ref var) => last_ident(var).map(ToString::to_string),
        luar_syn::FunctionName::Method(_, ref name) => Some(name.to_string()),
    };

    let meta = CodeMeta {
        arg_count: ArgumentCount::Known(decl.args.len().try_into().unwrap()),
        const_strings: state.strings,
        label_mappings: state.label_alloc.into_mappings(),
        return_count,
        local_count: state.reg_alloc.into_used_register_count(),
        debug_name,
        kind: FunctionKind::DeOptimized,
    };

    CodeBlock {
        meta,
        instructions: state.instructions,
    }
}

fn last_ident(var: &Var) -> Option<&Ident> {
    match var {
        Var::Named(ref ident) => Some(ident),
        Var::PropertyAccess { ref property, .. } => Some(property),
        Var::MemberLookup { .. } => None,
    }
}

pub fn compile_dyn_wrapper(
    arg_count: ArgumentCount,
    return_count: ReturnCount,
    local_block_id: LocalBlockID,
    debug_name: String,
) -> CodeBlock {
    use Instruction::*;

    let mut instructions = Vec::new();

    if let ArgumentCount::Known(arg_count) = arg_count {
        for i in 0..arg_count {
            instructions.push(LdaProt(ArgumentRegisterID(i)));
            instructions.push(StrRD(ArgumentRegisterID(i)));
        }
    }

    instructions.push(ConstC(local_block_id));
    instructions.push(TypedCall);

    if let ReturnCount::Constant(return_count) = return_count {
        let return_count: u32 = return_count.try_into().unwrap();
        instructions.push(ConstI(return_count as i32));
        instructions.push(StrVC);
    }

    instructions.push(Ret);

    CodeBlock {
        instructions,
        meta: CodeMeta {
            arg_count,
            return_count,
            debug_name: Some(debug_name),
            ..Default::default()
        },
    }
}

fn alias_arguments(args: &Vec<Ident>, state: &mut LocalScopeCompilationState) {
    use Instruction::*;

    let arg_count = args.len().try_into().unwrap();
    let locals = state.reg().alloc_count(DataType::Dynamic, arg_count);
    for (ident, i) in args.iter().cloned().zip(0..) {
        state.push_instr(LdaRD(ArgumentRegisterID(i)));
        state.push_instr(StrLD(locals.at(i)));
        state.define_local(ident.into(), locals.at(i));
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ids::{ArgumentRegisterID, LocalRegisterID, StringID},
        keyed_vec::keyed_vec,
        machine::CodeBlock,
        meta::{reg_count, CodeMeta},
        ops::Instruction,
        GlobalValues, LuaError,
    };

    use super::compile_function;

    use Instruction::*;

    macro_rules! test_instruction_output {
        ($name: ident, $code: expr, $fn_name: ident, $instr: expr) => {
            #[test]
            fn $name() -> Result<(), LuaError> {
                let function = luar_syn::lua_parser::function_declaration($code)?;
                let CodeBlock { meta, instructions } =
                    compile_function(&function, &mut GlobalValues::default());

                assert_eq!(
                    meta,
                    CodeMeta {
                        arg_count: 0.into(),
                        return_count: 1.into(),
                        debug_name: Some(stringify!($fn_name).to_string()),
                        ..Default::default()
                    }
                );

                assert_eq!(instructions, $instr);

                Ok(())
            }
        };
    }

    test_instruction_output!(
        compile_return_nil_fn,
        "function foo()
            return nil
        end",
        foo,
        vec![ConstN, StrRD(ArgumentRegisterID(0)), Ret]
    );

    test_instruction_output!(
        compile_return_int_fn,
        "function foo()
            return 42
        end",
        foo,
        vec![ConstI(42), WrapI, StrRD(ArgumentRegisterID(0)), Ret]
    );

    test_instruction_output!(
        compile_return_float_fn,
        "function foo()
            return 42.2
        end",
        foo,
        vec![ConstF(42.2), WrapF, StrRD(ArgumentRegisterID(0)), Ret]
    );

    #[test]
    fn compile_return_str_fn() -> Result<(), LuaError> {
        let function = luar_syn::lua_parser::function_declaration(
            "function foo()
                return 'hello'
            end",
        )?;
        let CodeBlock { meta, instructions } =
            compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            CodeMeta {
                arg_count: 0.into(),
                const_strings: keyed_vec!["hello".into()],
                return_count: 1.into(),
                debug_name: Some("foo".to_owned()),
                ..Default::default()
            }
        );

        use Instruction::*;
        assert_eq!(
            instructions,
            vec![
                ConstS(StringID(0)),
                WrapS,
                StrRD(ArgumentRegisterID(0)),
                Ret,
            ]
        );

        Ok(())
    }

    #[test]
    fn compile_empty_fn() -> Result<(), LuaError> {
        let function = luar_syn::lua_parser::function_declaration("function foo() end")?;
        let CodeBlock { meta, instructions } =
            compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            CodeMeta {
                arg_count: 0.into(),
                return_count: 0.into(),
                debug_name: Some("foo".to_owned()),
                ..Default::default()
            }
        );

        use Instruction::*;
        assert_eq!(instructions, vec![Ret]);

        Ok(())
    }

    #[test]
    fn compile_empty_empty_return_fn() -> Result<(), LuaError> {
        let function = luar_syn::lua_parser::function_declaration("function foo() return end")?;
        let CodeBlock { meta, instructions } =
            compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            CodeMeta {
                arg_count: 0.into(),
                return_count: 0.into(),
                debug_name: Some("foo".to_owned()),
                ..Default::default()
            }
        );

        use Instruction::*;
        assert_eq!(instructions, vec![Ret]);

        Ok(())
    }

    #[test]
    fn compile_simple_function() -> Result<(), LuaError> {
        let function = luar_syn::lua_parser::function_declaration(
            "function foo(a, b)
                return a + b
            end",
        )?;
        let CodeBlock { meta, instructions } =
            compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            CodeMeta {
                arg_count: 2.into(),
                return_count: 1.into(),
                local_count: reg_count! { D: 4 },
                debug_name: Some("foo".to_owned()),
                ..Default::default()
            }
        );

        use Instruction::*;
        assert_eq!(
            instructions,
            vec![
                LdaRD(ArgumentRegisterID(0)),
                StrLD(LocalRegisterID(0)),
                LdaRD(ArgumentRegisterID(1)),
                StrLD(LocalRegisterID(1)),

                LdaLD(LocalRegisterID(0)),
                StrLD(LocalRegisterID(2)),

                LdaLD(LocalRegisterID(1)),
                StrLD(LocalRegisterID(3)),

                LdaLD(LocalRegisterID(2)),
                DAddL(LocalRegisterID(3)),
                StrRD(ArgumentRegisterID(0)),
                Ret
            ]
        );

        Ok(())
    }
}
