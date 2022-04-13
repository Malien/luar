use crate::reggie::{
    compiler::{expr::compile_expr, FunctionCompilationState, LocalFnCompState},
    ids::ArgumentRegisterID,
    machine::{CodeBlock, GlobalValues},
    meta::{CodeMeta, MetaCount},
    ops::Instruction,
};
use crate::syn;

pub fn compile_function(
    decl: &syn::FunctionDeclaration,
    global_values: &mut GlobalValues,
) -> CodeBlock {
    use Instruction::*;
    let return_count = decl.body.ret.as_ref().map(|ret| ret.0.len()).unwrap_or(0);
    let mut state = FunctionCompilationState::with_args(decl.args.iter().cloned(), global_values);
    let mut root_scope = LocalFnCompState::new(&mut state);

    for statement in &decl.body.statements {
        todo!("Compiling statement \"{}\" in function body is not implemented yet", statement);
    }

    if let Some(syn::Return(exprs)) = &decl.body.ret {
        if let Some(expr) = exprs.first() {
            compile_expr(expr, &mut root_scope);
            root_scope.push_instr(StrRD(ArgumentRegisterID(0)));
        }
    }

    state.instructions.push(Ret);

    let meta = CodeMeta {
        arg_count: decl.args.len().into(),
        const_strings: state.strings,
        label_mappings: vec![],
        return_count: MetaCount::Known(return_count),
        local_count: state.alloc.into_used_register_count(),
    };

    CodeBlock {
        meta,
        instructions: state.instructions,
    }
}

#[cfg(test)]
mod test {
    use crate::reggie::{
        ids::{ArgumentRegisterID, LocalRegisterID, StringID},
        machine::{CodeBlock, GlobalValues},
        meta::{LocalRegCount, MetaCount},
        ops::Instruction,
    };
    use crate::{error::LuaError, reggie::meta::CodeMeta, syn};

    use super::compile_function;

    use Instruction::*;

    macro_rules! test_instruction_output {
        ($name: ident, $code: expr, $instr: expr) => {
            #[test]
            fn $name() -> Result<(), LuaError> {
                let function = syn::lua_parser::function_declaration($code)?;
                let CodeBlock { meta, instructions } =
                    compile_function(&function, &mut GlobalValues::default());

                assert_eq!(
                    meta,
                    CodeMeta {
                        arg_count: 0.into(),
                        const_strings: vec![],
                        label_mappings: vec![],
                        return_count: 1.into(),
                        local_count: LocalRegCount::default(),
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
        let function = syn::lua_parser::function_declaration(
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
                const_strings: vec!["hello".to_string()],
                label_mappings: vec![],
                return_count: MetaCount::Known(1),
                local_count: LocalRegCount::default(),
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
        let function = syn::lua_parser::function_declaration("function foo() end")?;
        let CodeBlock { meta, instructions } =
            compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            CodeMeta {
                arg_count: 0.into(),
                const_strings: vec![],
                label_mappings: vec![],
                return_count: MetaCount::Known(0),
                local_count: LocalRegCount::default(),
            }
        );

        use Instruction::*;
        assert_eq!(instructions, vec![Ret]);

        Ok(())
    }

    #[test]
    fn compile_empty_empty_return_fn() -> Result<(), LuaError> {
        let function = syn::lua_parser::function_declaration("function foo() return end")?;
        let CodeBlock { meta, instructions } =
            compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            CodeMeta {
                arg_count: 0.into(),
                const_strings: vec![],
                label_mappings: vec![],
                return_count: MetaCount::Known(0),
                local_count: LocalRegCount::default(),
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
                let function = syn::lua_parser::function_declaration($fn)?;
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
        let function = syn::lua_parser::function_declaration(
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
                const_strings: vec![],
                label_mappings: vec![],
                return_count: MetaCount::Known(1),
                local_count: LocalRegCount {
                    d: 1,
                    ..Default::default()
                },
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
            const_strings: vec![],
            label_mappings: vec![],
            return_count: MetaCount::Known(1),
            local_count: LocalRegCount {
                d: 1,
                ..Default::default()
            },
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
            const_strings: vec![],
            label_mappings: vec![],
            return_count: MetaCount::Known(1),
            local_count: LocalRegCount {
                d: 1,
                ..Default::default()
            },
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
            const_strings: vec![],
            label_mappings: vec![],
            return_count: MetaCount::Known(1),
            local_count: LocalRegCount {
                d: 1,
                ..Default::default()
            },
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
        let function = syn::lua_parser::function_declaration(
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
                const_strings: vec![],
                label_mappings: vec![],
                return_count: MetaCount::Known(1),
                local_count: LocalRegCount {
                    d: 1,
                    ..Default::default()
                },
            }
        );

        use Instruction::*;
        assert_eq!(
            instructions,
            vec![
                LdaRD(ArgumentRegisterID(0)),
                StrLD(LocalRegisterID(0)),
                LdaRD(ArgumentRegisterID(1)),
                DAddL(LocalRegisterID(0)),
                StrRD(ArgumentRegisterID(0)),
                Ret,
            ]
        );

        Ok(())
    }
}
