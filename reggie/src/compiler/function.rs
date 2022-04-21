use luar_lex::Ident;
use luar_syn::FunctionDeclaration;

use crate::{
    compiler::{
        compile_statement, ret::compile_ret, FunctionCompilationState, LocalScopeCompilationState,
    },
    ids::{ArgumentRegisterID, LocalBlockID},
    machine::{CodeBlock, GlobalValues},
    meta::{ArgumentCount, CodeMeta, ReturnCount},
    ops::Instruction,
};

pub struct CompiledFunction {
    wrapper: CodeBlock,
    optimized: Option<CodeBlock>,
}

pub fn compile_function(decl: &FunctionDeclaration, global_values: &mut GlobalValues) -> CodeBlock {
    use Instruction::*;
    let mut state = FunctionCompilationState::with_args(decl.args.iter().cloned(), global_values);
    let mut root_scope = LocalScopeCompilationState::new(&mut state);

    alias_arguments(&decl.args, &mut root_scope);

    for statement in &decl.body.statements {
        compile_statement(statement, &mut root_scope);
    }

    if let Some(ret) = &decl.body.ret {
        compile_ret(ret, &mut root_scope);
    } else {
        root_scope.push_instr(Ret);
    }

    let meta = CodeMeta {
        arg_count: ArgumentCount::Known(decl.args.len().try_into().unwrap()),
        const_strings: state.strings,
        label_mappings: state.label_alloc.into_mappings(),
        return_count: state
            .return_count
            .into_return_count()
            .unwrap_or(ReturnCount::Constant(0)),
        local_count: state.reg_alloc.into_used_register_count(),
    };

    CodeBlock {
        meta,
        instructions: state.instructions,
    }
}

pub fn compile_dyn_wrapper(
    arg_count: ArgumentCount,
    return_count: ReturnCount,
    local_block_id: LocalBlockID,
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
            ..Default::default()
        },
    }
}

fn alias_arguments(args: &Vec<Ident>, state: &mut LocalScopeCompilationState) {
    use Instruction::*;

    let arg_count = args.len().try_into().unwrap();
    let locals = state.reg().alloc_dyn_count(arg_count);
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
        machine::{CodeBlock, GlobalValues},
        meta::{CodeMeta, LocalRegCount},
        ops::Instruction,
        LuaError,
    };

    use super::compile_function;

    use Instruction::*;

    macro_rules! test_instruction_output {
        ($name: ident, $code: expr, $instr: expr) => {
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
        vec![ConstN, StrRD(ArgumentRegisterID(0)), Ret]
    );

    test_instruction_output!(
        compile_return_int_fn,
        "function foo()
            return 42
        end",
        vec![ConstI(42), WrapI, StrRD(ArgumentRegisterID(0)), Ret]
    );

    test_instruction_output!(
        compile_return_float_fn,
        "function foo()
            return 42.2
        end",
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
                const_strings: keyed_vec!["hello".to_string()],
                return_count: 1.into(),
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
                ..Default::default()
            }
        );

        use Instruction::*;
        assert_eq!(instructions, vec![Ret]);

        Ok(())
    }

    macro_rules! test_compilation {
        ($name: ident, $fn:expr, $meta:expr, $instr:expr) => {
            #[test]
            fn $name() -> Result<(), LuaError> {
                let function = luar_syn::lua_parser::function_declaration($fn)?;
                let CodeBlock { meta, instructions } =
                    compile_function(&function, &mut GlobalValues::default());
                assert_eq!(meta, $meta);
                assert_eq!(instructions, $instr);
                Ok(())
            }
        };
    }

    #[test]
    fn compile_add_two_constants_fn() -> Result<(), LuaError> {
        let function = luar_syn::lua_parser::function_declaration(
            "function foo()
                return 1 + 2
            end",
        )?;
        let CodeBlock { meta, instructions } =
            compile_function(&function, &mut GlobalValues::default());
        assert_eq!(
            meta,
            CodeMeta {
                arg_count: 0.into(),
                return_count: 1.into(),
                local_count: LocalRegCount {
                    d: 1,
                    ..Default::default()
                },
                ..Default::default()
            }
        );

        use Instruction::*;
        assert_eq!(
            instructions,
            vec![
                ConstI(1),
                WrapI,
                StrLD(LocalRegisterID(0)),
                ConstI(2),
                WrapI,
                DAddL(LocalRegisterID(0)),
                StrRD(ArgumentRegisterID(0)),
                Ret
            ]
        );

        Ok(())
    }

    test_compilation!(
        compile_sub_two_constants_fn,
        "function foo()
            return 1 - 2
        end",
        CodeMeta {
            arg_count: 0.into(),
            return_count: 1.into(),
            local_count: LocalRegCount {
                d: 1,
                ..Default::default()
            },
            ..Default::default()
        },
        [
            ConstI(1),
            WrapI,
            StrLD(LocalRegisterID(0)),
            ConstI(2),
            WrapI,
            DSubL(LocalRegisterID(0)),
            StrRD(ArgumentRegisterID(0)),
            Ret
        ]
    );

    test_compilation!(
        compile_mul_two_constants_fn,
        "function foo()
            return 1 * 2
        end",
        CodeMeta {
            arg_count: 0.into(),
            return_count: 1.into(),
            local_count: LocalRegCount {
                d: 1,
                ..Default::default()
            },
            ..Default::default()
        },
        [
            ConstI(1),
            WrapI,
            StrLD(LocalRegisterID(0)),
            ConstI(2),
            WrapI,
            DMulL(LocalRegisterID(0)),
            StrRD(ArgumentRegisterID(0)),
            Ret
        ]
    );

    test_compilation!(
        compile_div_two_constants_fn,
        "function foo()
            return 1 / 2
        end",
        CodeMeta {
            arg_count: 0.into(),
            return_count: 1.into(),
            local_count: LocalRegCount {
                d: 1,
                ..Default::default()
            },
            ..Default::default()
        },
        [
            ConstI(1),
            WrapI,
            StrLD(LocalRegisterID(0)),
            ConstI(2),
            WrapI,
            DDivL(LocalRegisterID(0)),
            StrRD(ArgumentRegisterID(0)),
            Ret
        ]
    );

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
                local_count: LocalRegCount {
                    d: 3,
                    ..Default::default()
                },
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
                DAddL(LocalRegisterID(2)),
                StrRD(ArgumentRegisterID(0)),
                Ret,
            ]
        );

        Ok(())
    }
}
