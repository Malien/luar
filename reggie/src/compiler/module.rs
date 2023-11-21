use luar_syn::{Chunk, FunctionName, Return, Var};

use crate::{
    global_values::GlobalValues,
    ids::LocalBlockID,
    keyed_vec::KeyedVec,
    machine::CodeBlock,
    meta::{ArgumentCount, CodeMeta, FunctionKind, ReturnCount},
    ops::Instruction,
};

use super::{
    compile_dyn_wrapper, compile_function, compile_statement, ret::compile_ret,
    return_traversal::return_traverse_module, FunctionCompilationState, LocalScopeCompilationState,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledModule {
    pub blocks: KeyedVec<LocalBlockID, CodeBlock>,
    pub top_level: CodeBlock,
}

impl std::fmt::Display for CompiledModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (local_block_id, block) in &self.blocks {
            writeln!(f, "local block {} {}", local_block_id.0, block)?;
        }
        writeln!(f, "root block {}", self.top_level)
    }
}

pub fn compile_module(
    module: &luar_syn::Module,
    global_values: &mut GlobalValues,
) -> CompiledModule {
    let return_count = return_traverse_module(module);
    let mut state = FunctionCompilationState::new(global_values, return_count);
    let mut root_scope = LocalScopeCompilationState::new(&mut state);
    let mut blocks = KeyedVec::new();

    for chunk in &module.chunks {
        match chunk {
            Chunk::FnDecl(decl) => {
                compile_function_declaration(&mut root_scope, decl, &mut blocks);
            }
            Chunk::Statement(statement) => {
                compile_statement(statement, &mut root_scope);
            }
        };
    }

    let empty_ret = Return(vec![]);
    let ret = module.ret.as_ref().unwrap_or(&empty_ret);
    compile_ret(ret, &mut root_scope);

    CompiledModule {
        blocks,
        top_level: CodeBlock {
            instructions: state.instructions,
            meta: CodeMeta {
                arg_count: ArgumentCount::Known(0),
                local_count: state.reg_alloc.into_used_register_count(),
                return_count,
                label_mappings: state.label_alloc.into_mappings(),
                const_strings: state.strings,
                debug_name: Some("<module root>".to_owned()),
                kind: FunctionKind::DeOptimized,
            },
        },
    }
}

fn compile_function_declaration(
    root_scope: &mut LocalScopeCompilationState,
    decl: &luar_syn::FunctionDeclaration,
    blocks: &mut KeyedVec<LocalBlockID, CodeBlock>,
) {
    let global_values = root_scope.global_values();
    let func = compile_function(decl, global_values);

    let func_to_save = if needs_wrapper(&func.meta) {
        wrap_function(func, blocks)
    } else {
        func
    };

    let func_id = blocks.push(func_to_save);

    match &decl.name {
        FunctionName::Plain(Var::Named(name)) => {
            let cell = global_values.cell_for_name(name.as_ref());
            root_scope.push_instr(Instruction::ConstC(func_id));
            root_scope.push_instr(Instruction::WrapC);
            root_scope.push_instr(Instruction::StrDGl(cell));
        }
        FunctionName::Plain(var) => todo!("Error compiling function declaration of {var}. Compilation of complex table function declaration is not implemented"),
        FunctionName::Method(base, name) => todo!("Error compiling function declaration of {base}:{name}. Compilation of method function declaration is not implemented"),
    }
}

fn wrap_function(func: CodeBlock, blocks: &mut KeyedVec<LocalBlockID, CodeBlock>) -> CodeBlock {
    let return_count = func.meta.return_count;
    let arg_count = func.meta.arg_count;
    let func_id = blocks.next_key();
    let debug_name = if let Some(ref wrapee_name) = func.meta.debug_name {
        format!("<dyn wrapper for function {}>", wrapee_name)
    } else {
        format!("<dyn wrapper for local block {}>", func_id.0)
    };
    blocks.push(func);
    compile_dyn_wrapper(arg_count, return_count, func_id, debug_name)
}

fn needs_wrapper(meta: &CodeMeta) -> bool {
    !matches!(
        meta.arg_count,
        ArgumentCount::Known(0) | ArgumentCount::Unknown
    ) || matches!(meta.return_count, ReturnCount::Constant(_))
}

#[cfg(test)]
mod test {
    use nonzero_ext::nonzero;

    use super::compile_module;
    use crate::{
        compiler::compile_function,
        ids::{ArgumentRegisterID, JmpLabel, LocalBlockID, StringID},
        keyed_vec::keyed_vec,
        machine::CodeBlock,
        meta::{CodeMeta, LocalRegCount, ReturnCount},
        ops::Instruction,
        GlobalValues, LuaError,
    };

    use luar_syn::lua_parser;
    use Instruction::*;

    macro_rules! test_instruction_output {
        ($name: ident, $code: expr, $instr: expr) => {
            #[test]
            fn $name() -> Result<(), LuaError> {
                let module = luar_syn::lua_parser::module($code)?;
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
        let module = luar_syn::lua_parser::module("return 'hello'")?;
        let compiled_module = compile_module(&module, &mut GlobalValues::default());

        assert_eq!(
            compiled_module.top_level.meta.local_count,
            LocalRegCount::default()
        );
        assert_eq!(
            compiled_module.top_level.meta.const_strings,
            keyed_vec!["hello".to_string()]
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
        let module = luar_syn::lua_parser::module("")?;
        let compiled_module = compile_module(&module, &mut GlobalValues::default());

        assert_eq!(
            compiled_module.top_level.meta.local_count,
            LocalRegCount::default()
        );
        assert_eq!(
            compiled_module.top_level.meta.return_count,
            ReturnCount::Constant(0)
        );
        use Instruction::*;
        assert_eq!(compiled_module.top_level.instructions, vec![Ret]);

        Ok(())
    }

    #[test]
    fn empty_module_and_module_with_empty_return_compiles_identically() -> Result<(), LuaError> {
        let ret_module = luar_syn::lua_parser::module("return")?;
        let empty_module = luar_syn::lua_parser::module("")?;
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
                let module = luar_syn::lua_parser::module($fn)?;
                let compiled_module = compile_module(&module, &mut GlobalValues::default());
                assert_eq!(compiled_module.top_level, $meta);
                Ok(())
            }
        };
    }

    #[test]
    fn compile_function_declaration() -> Result<(), LuaError> {
        let module = luar_syn::lua_parser::module("function foo() return 42 end")?;
        let function_decl =
            luar_syn::lua_parser::function_declaration("function foo() return 42 end")?;
        let mut global_values = GlobalValues::default();
        let module = compile_module(&module, &mut global_values);
        let func = compile_function(&function_decl, &mut GlobalValues::default());

        assert_eq!(module.top_level.meta.return_count, ReturnCount::Constant(0));
        assert_eq!(module.top_level.meta.local_count, LocalRegCount::default());
        assert_eq!(
            module.top_level.instructions,
            vec![
                ConstC(LocalBlockID(1)),
                WrapC,
                StrDGl(global_values.cell_for_name("foo")),
                Ret
            ]
        );
        assert!(module.blocks.slice().contains(&func));

        Ok(())
    }

    test_compilation!(
        compile_simple_if,
        "if nil then return 4 end return 5",
        CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                return_count: 1.into(),
                label_mappings: keyed_vec![7],
                debug_name: Some("<module root>".to_owned()),
                ..Default::default()
            },
            instructions: vec![
                ConstN,
                NilTest,
                JmpEQ(JmpLabel(0)),
                ConstI(4),
                WrapI,
                StrRD(ArgumentRegisterID(0)),
                Ret,
                Label,
                ConstI(5),
                WrapI,
                StrRD(ArgumentRegisterID(0)),
                Ret
            ]
        }
    );

    #[test]
    fn correct_return_count() {
        use ReturnCount::*;

        let expectations = [
            ("", Constant(0)),
            ("return", Constant(0)),
            ("return 1", Constant(1)),
            ("return 1,2,3", Constant(3)),
            ("return func()", Unbounded),
            ("return 1,2,func()", MinBounded(nonzero!(2u16))),
            ("if nil then return end", Constant(0)),
            (
                "if nil then return end return 5",
                Bounded {
                    min: 0,
                    max: nonzero!(1u16),
                },
            ),
            ("if nil then return 1,2,3 end return func()", Unbounded),
            (
                "if nil then return 1,2,func() end return 1,func()",
                MinBounded(nonzero!(1u16)),
            ),
        ];

        let mut global_values = GlobalValues::default();
        for (module_str, return_count) in expectations {
            let module = lua_parser::module(module_str).unwrap();
            let compiled_module = compile_module(&module, &mut global_values);
            assert_eq!(
                compiled_module.top_level.meta.return_count, return_count,
                "Expected module \"{}\" to have return count of {:?}, got {:?}",
                module_str, return_count, compiled_module.top_level.meta.return_count
            );
        }
    }
}
