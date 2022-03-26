use std::collections::HashMap;

use crate::{
    reggie::ids::{ArgumentRegisterID, StringID},
    syn,
};

use super::{
    fn_meta::{FnMeta, LocalRegCount, ReturnCount},
    ops::Instruction,
};

#[derive(Debug, Clone, Copy, Default)]
struct RegisterAllocator {
    max: LocalRegCount,
    used: LocalRegCount,
}

#[derive(Debug, Clone, Default)]
struct FunctionCompilationState {
    alloc: RegisterAllocator,
    strings: Vec<String>,
    instructions: Vec<Instruction>,
}

pub fn compile_function(decl: &syn::FunctionDeclaration) -> (FnMeta, Vec<Instruction>) {
    use Instruction::*;
    let arg_mapping: HashMap<&str, usize> = decl.args.iter().map(AsRef::as_ref).zip(0..).collect();

    let return_count = decl.body.ret.as_ref().map(|ret| ret.0.len()).unwrap_or(0);
    let local_count = LocalRegCount::default();
    let mut state = FunctionCompilationState::default();

    // for statement in decl.body.statements {
    //     match statement {
    //         syn::Statement::
    //     }
    // }

    if let Some(syn::Return(exprs)) = &decl.body.ret {
        if let Some(expr) = exprs.first() {
            compile_expr(expr, &mut state);
            state.instructions.push(StrRD(ArgumentRegisterID(0)))
        }
    }

    state.instructions.push(Ret);

    let meta = FnMeta {
        arg_count: decl.args.len(),
        const_strings: state.strings,
        label_mappings: vec![],
        return_count: ReturnCount::Known(return_count),
        local_count,
    };
    return (meta, state.instructions);
}

fn compile_expr(expr: &syn::Expression, state: &mut FunctionCompilationState) {
    use Instruction::*;
    let FunctionCompilationState {
        instructions,
        strings,
        ..
    } = state;

    match expr {
        syn::Expression::Nil => {
            instructions.push(ConstN);
        }
        syn::Expression::Number(num) if num.is_integer() => {
            instructions.push(ConstI(num.as_i32()));
            instructions.push(WrapI);
        }
        syn::Expression::Number(num) => {
            instructions.push(ConstF(num.as_f64()));
            instructions.push(WrapF);
        }
        syn::Expression::String(str) => {
            let str_idx = strings.len();
            strings.push(str.0.clone());
            instructions.push(ConstS(StringID(str_idx.try_into().unwrap())));
            instructions.push(WrapS);
        }
        // syn::Expression::BinaryOperator { lhs: (), op: (), rhs: () }
        _ => todo!(),
    }
}

#[cfg(test)]
mod test {
    use crate::reggie::fn_meta::{LocalRegCount, ReturnCount};
    use crate::reggie::ids::{ArgumentRegisterID, StringID};
    use crate::reggie::ops::Instruction;
    use crate::{error::LuaError, reggie::fn_meta::FnMeta, syn};

    use super::compile_function;

    use Instruction::*;

    macro_rules! test_instruction_output {
        ($name: ident, $code: expr, $instr: expr) => {
            #[test]
            fn $name() -> Result<(), LuaError> {
                let function = syn::lua_parser::function_declaration($code)?;
                let (meta, instructions) = compile_function(&function);

                assert_eq!(
                    meta,
                    FnMeta {
                        arg_count: 0,
                        const_strings: vec![],
                        label_mappings: vec![],
                        return_count: ReturnCount::Known(1),
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
        let (meta, instructions) = compile_function(&function);

        assert_eq!(
            meta,
            FnMeta {
                arg_count: 0,
                const_strings: vec!["hello".to_string()],
                label_mappings: vec![],
                return_count: ReturnCount::Known(1),
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
        let (meta, instructions) = compile_function(&function);

        assert_eq!(
            meta,
            FnMeta {
                arg_count: 0,
                const_strings: vec![],
                label_mappings: vec![],
                return_count: ReturnCount::Known(0),
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
        let (meta, instructions) = compile_function(&function);

        assert_eq!(
            meta,
            FnMeta {
                arg_count: 0,
                const_strings: vec![],
                label_mappings: vec![],
                return_count: ReturnCount::Known(0),
                local_count: LocalRegCount::default(),
            }
        );

        use Instruction::*;
        assert_eq!(instructions, vec![Ret]);

        Ok(())
    }


    #[test]
    fn compile_simple_function() -> Result<(), LuaError> {
        let function = syn::lua_parser::function_declaration(
            "function foo(a, b)
                return a + b
            end",
        )?;
        let (meta, instructions) = compile_function(&function);

        assert_eq!(
            meta,
            FnMeta {
                arg_count: 2,
                const_strings: vec![],
                label_mappings: vec![],
                return_count: ReturnCount::Known(1),
                local_count: LocalRegCount::default(),
            }
        );

        use Instruction::*;
        assert_eq!(
            instructions,
            vec![
                LdaRD(ArgumentRegisterID(0)),
                DAddR(ArgumentRegisterID(1)),
                StrRD(ArgumentRegisterID(0)),
                Ret,
            ]
        );

        Ok(())
    }
}
