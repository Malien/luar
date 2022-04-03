use crate::{
    reggie::{
        machine::{CodeBlock, GlobalValues},
        meta::{CodeMeta, MetaCount},
        ops::Instruction, ids::ArgumentRegisterID,
    },
    syn,
};

use super::{expr::compile_expr, FunctionCompilationState, LocalFnCompState};

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledModule {
    pub blocks: Vec<CodeBlock>,
    pub top_level: CodeBlock,
}

pub fn compile_module(module: &syn::Module, global_values: &mut GlobalValues) -> CompiledModule {
    let mut functions = Vec::new();
    let mut statements = Vec::new();

    let return_count = module.ret.as_ref().map(|ret| ret.0.len()).unwrap_or(0);
    let mut state = FunctionCompilationState::new(global_values);
    let mut root_scope = LocalFnCompState::new(&mut state);

    for chunk in &module.chunks {
        match chunk {
            syn::Chunk::FnDecl(func) => functions.push(func),
            syn::Chunk::Statement(statement) => statements.push(statement),
        };
    }

    if let Some(syn::Return(expressions)) = &module.ret {
        if let Some(expr) = expressions.first() {
            compile_expr(expr, &mut root_scope);
            root_scope.push_instr(Instruction::StrRD(ArgumentRegisterID(0)));
        }
    }

    root_scope.push_instr(Instruction::Ret);

    CompiledModule {
        blocks: vec![],
        top_level: CodeBlock {
            instructions: state.instructions,
            meta: CodeMeta {
                arg_count: MetaCount::Known(0),
                local_count: state.alloc.into_used_register_count(),
                return_count: MetaCount::Known(return_count),
                label_mappings: vec![],
                const_strings: state.strings,
            },
        },
    }
}

#[cfg(test)]
mod test {
    use super::compile_module;
    use crate::{
        error::LuaError,
        reggie::{
            ids::{ArgumentRegisterID, LocalRegisterID, StringID},
            machine::{CodeBlock, GlobalValues},
            meta::{CodeMeta, LocalRegCount, MetaCount},
            ops::Instruction,
        },
        syn,
    };

    use Instruction::*;

    macro_rules! test_instruction_output {
        ($name: ident, $code: expr, $instr: expr) => {
            #[test]
            fn $name() -> Result<(), LuaError> {
                let module = syn::lua_parser::module($code)?;
                let compiled_module = compile_module(&module, &mut GlobalValues::default());

                assert_eq!(compiled_module.top_level.meta.return_count, 1.into());
                assert_eq!(compiled_module.top_level.instructions, $instr);

                Ok(())
            }
        };
    }

    test_instruction_output!(
        compile_return_nil,
        "return nil",
        vec![ConstN, StrRD(ArgumentRegisterID(0)), Ret]
    );

    test_instruction_output!(
        compile_return_int,
        "return 42",
        vec![ConstI(42), WrapI, StrRD(ArgumentRegisterID(0)), Ret]
    );

    test_instruction_output!(
        compile_return_float,
        "return 42.2",
        vec![ConstF(42.2), WrapF, StrRD(ArgumentRegisterID(0)), Ret]
    );

    #[test]
    fn compile_return_str() -> Result<(), LuaError> {
        let module = syn::lua_parser::module("return 'hello'")?;
        let compiled_module = compile_module(&module, &mut GlobalValues::default());

        assert_eq!(
            compiled_module.top_level.meta.local_count,
            LocalRegCount::default()
        );
        assert_eq!(
            compiled_module.top_level.meta.const_strings,
            vec!["hello".to_string()]
        );

        use Instruction::*;
        assert_eq!(
            compiled_module.top_level.instructions,
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
    fn compile_empty() -> Result<(), LuaError> {
        let module = syn::lua_parser::module("")?;
        let compiled_module = compile_module(&module, &mut GlobalValues::default());

        assert_eq!(
            compiled_module.top_level.meta.local_count,
            LocalRegCount::default()
        );
        assert_eq!(
            compiled_module.top_level.meta.return_count,
            MetaCount::Known(0)
        );
        use Instruction::*;
        assert_eq!(compiled_module.top_level.instructions, vec![Ret]);

        Ok(())
    }

    #[test]
    fn empty_module_and_module_with_empty_return_compiles_identically() -> Result<(), LuaError> {
        let ret_module = syn::lua_parser::module("return")?;
        let empty_module = syn::lua_parser::module("")?;
        assert_eq!(
            compile_module(&ret_module, &mut GlobalValues::default()),
            compile_module(&empty_module, &mut GlobalValues::default())
        );

        Ok(())
    }

    macro_rules! test_compilation {
        ($name: ident, $fn:expr, $meta:expr) => {
            #[test]
            fn $name() -> Result<(), LuaError> {
                let module = syn::lua_parser::module($fn)?;
                let compiled_module = compile_module(&module, &mut GlobalValues::default());
                assert_eq!(compiled_module.top_level, $meta);
                Ok(())
            }
        };
    }

    #[test]
    fn compile_add_two_constants() -> Result<(), LuaError> {
        let module = syn::lua_parser::module("return 1 + 2")?;
        let module = compile_module(&module, &mut GlobalValues::default());

        assert_eq!(module.top_level.meta.return_count, MetaCount::Known(1));
        assert_eq!(
            module.top_level.meta.local_count,
            LocalRegCount {
                d: 1,
                ..Default::default()
            }
        );

        use Instruction::*;
        assert_eq!(
            module.top_level.instructions,
            [
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
        compile_sub_two_constants,
        "return 1 - 2",
        CodeBlock {
            instructions: vec![
                ConstI(1),
                WrapI,
                StrLD(LocalRegisterID(0)),
                ConstI(2),
                WrapI,
                DSubL(LocalRegisterID(0)),
                StrRD(ArgumentRegisterID(0)),
                Ret
            ],
            meta: CodeMeta {
                arg_count: MetaCount::Known(0),
                const_strings: vec![],
                label_mappings: vec![],
                return_count: MetaCount::Known(1),
                local_count: LocalRegCount {
                    d: 1,
                    ..Default::default()
                },
            }
        }
    );

    test_compilation!(
        compile_mul_two_constants,
        "return 1 * 2",
        CodeBlock {
            instructions: vec![
                ConstI(1),
                WrapI,
                StrLD(LocalRegisterID(0)),
                ConstI(2),
                WrapI,
                DMulL(LocalRegisterID(0)),
                StrRD(ArgumentRegisterID(0)),
                Ret
            ],
            meta: CodeMeta {
                arg_count: MetaCount::Known(0),
                const_strings: vec![],
                label_mappings: vec![],
                return_count: MetaCount::Known(1),
                local_count: LocalRegCount {
                    d: 1,
                    ..Default::default()
                },
            }
        }
    );

    test_compilation!(
        compile_div_two_constants,
        "return 1 / 2",
        CodeBlock {
            instructions: vec![
                ConstI(1),
                WrapI,
                StrLD(LocalRegisterID(0)),
                ConstI(2),
                WrapI,
                DDivL(LocalRegisterID(0)),
                StrRD(ArgumentRegisterID(0)),
                Ret
            ],
            meta: CodeMeta {
                arg_count: 0.into(),
                const_strings: vec![],
                label_mappings: vec![],
                return_count: MetaCount::Known(1),
                local_count: LocalRegCount {
                    d: 1,
                    ..Default::default()
                },
            }
        }
    );
}
