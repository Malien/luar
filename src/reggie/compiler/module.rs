use crate::{
    reggie::{
        ids::{ArgumentRegisterID, LocalBlockID},
        machine::{CodeBlock, GlobalValues},
        meta::{CodeMeta, MetaCount},
        ops::Instruction,
    },
    syn,
};

use super::{compile_function, expr::compile_expr, FunctionCompilationState, LocalFnCompState};

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledModule {
    pub blocks: Vec<CodeBlock>,
    pub top_level: CodeBlock,
}

pub fn compile_module(module: &syn::Module, global_values: &mut GlobalValues) -> CompiledModule {
    let mut blocks = Vec::new();

    let return_count = module.ret.as_ref().map(|ret| ret.0.len()).unwrap_or(0);
    let mut state = FunctionCompilationState::new(global_values);
    let mut root_scope = LocalFnCompState::new(&mut state);

    for chunk in &module.chunks {
        match chunk {
            syn::Chunk::FnDecl(decl) => {
                let global_values = root_scope.global_values();
                let func = compile_function(decl, global_values);
                let local_block_id = LocalBlockID(blocks.len().try_into().unwrap());
                blocks.push(func);
                match &decl.name {
                    syn::FunctionName::Plain(syn::Var::Named(name)) => {
                        let cell = global_values.cell_for_name(name.as_ref());
                        root_scope.push_instr(Instruction::ConstC(local_block_id));
                        root_scope.push_instr(Instruction::WrapC);
                        root_scope.push_instr(Instruction::StrDGl(cell));
                    }
                    _ => todo!(),
                }
            }
            syn::Chunk::Statement(statement) => todo!("Compiling statement \"{}\" is not implemented", statement),
        };
    }

    if let Some(syn::Return(expressions)) = &module.ret {
        if expressions.len() > 1 {
            todo!();
        }
        if let Some(expr) = expressions.first() {
            compile_expr(expr, &mut root_scope);
            root_scope.push_instr(Instruction::StrRD(ArgumentRegisterID(0)));
        }
    }

    root_scope.push_instr(Instruction::Ret);

    CompiledModule {
        blocks,
        top_level: CodeBlock {
            instructions: state.instructions,
            meta: CodeMeta {
                arg_count: MetaCount::Known(0),
                local_count: state.reg_alloc.into_used_register_count(),
                return_count: MetaCount::Known(return_count),
                label_mappings: state.label_alloc.into_mappings(),
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
            compiler::compile_function,
            ids::{ArgumentRegisterID, LocalRegisterID, StringID, LocalBlockID},
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

    #[test]
    fn compile_function_declaration() -> Result<(), LuaError> {
        let module = syn::lua_parser::module("function foo() return 42 end")?;
        let function_decl = syn::lua_parser::function_declaration("function foo() return 42 end")?;
        let mut global_values = GlobalValues::default();
        let module = compile_module(&module, &mut global_values);
        let func = compile_function(&function_decl, &mut GlobalValues::default());

        assert_eq!(module.top_level.meta.return_count, MetaCount::Known(0));
        assert_eq!(module.top_level.meta.local_count, LocalRegCount::default());
        assert_eq!(
            module.top_level.instructions,
            vec![
                ConstC(LocalBlockID(0)),
                WrapC,
                StrDGl(global_values.cell_for_name("foo")),
                Ret
            ]
        );
        assert_eq!(module.blocks, vec![func]);

        Ok(())
    }
}
