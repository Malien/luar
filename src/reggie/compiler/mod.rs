use std::collections::HashMap;

use crate::{
    reggie::ids::{ArgumentRegisterID, StringID},
    syn,
};

use super::{
    fn_meta::{FnMeta, LocalRegCount, MetaCount},
    ids::{GlobalCellID, LocalRegisterID},
    machine::GlobalValues,
    ops::Instruction,
};

#[derive(Debug, Clone, Copy, Default)]
struct RegisterAllocator {
    total: LocalRegCount,
    in_use: LocalRegCount,
}

impl RegisterAllocator {
    fn into_used_register_count(self) -> LocalRegCount {
        self.total
    }

    fn alloc_dyn(&mut self) -> LocalRegisterID {
        let reg_id = LocalRegisterID(self.in_use.d);
        self.in_use.d += 1;
        self.total.d = std::cmp::max(self.total.d, self.in_use.d);
        reg_id
    }

    fn free_dyn(&mut self) {
        self.in_use.d -= 1;
    }
}

#[derive(Debug, Clone, Default)]
struct LocalScope(HashMap<String, LocalRegisterID>);

#[derive(Debug, Clone, Default)]
struct ArgumentScope(HashMap<String, ArgumentRegisterID>);

#[derive(Debug)]
struct FunctionCompilationState<'a> {
    global_values: &'a mut GlobalValues,
    alloc: RegisterAllocator,
    strings: Vec<String>,
    instructions: Vec<Instruction>,
    arguments: ArgumentScope,
    scope_vars: Vec<LocalScope>,
}

impl<'a> FunctionCompilationState<'a> {
    fn new(global_values: &'a mut GlobalValues) -> Self {
        Self {
            global_values,
            alloc: Default::default(),
            strings: Default::default(),
            instructions: Default::default(),
            arguments: Default::default(),
            scope_vars: Default::default(),
        }
    }

    fn with_args(
        args: impl IntoIterator<Item = impl Into<String>>,
        global_values: &'a mut GlobalValues,
    ) -> Self {
        Self {
            global_values,
            alloc: Default::default(),
            strings: Default::default(),
            instructions: Default::default(),
            arguments: ArgumentScope(
                args.into_iter()
                    .map(Into::into)
                    .zip((0..).into_iter().map(ArgumentRegisterID))
                    .collect(),
            ),
            scope_vars: Default::default(),
        }
    }
}

#[derive(Debug)]
struct LocalFnCompState<'a, 'b> {
    func_state: &'a mut FunctionCompilationState<'b>,
    scope: usize,
}

enum VarLookup {
    Argument(ArgumentRegisterID),
    Local(LocalRegisterID),
    GlobalCell(GlobalCellID),
}

impl<'a, 'b> LocalFnCompState<'a, 'b> {
    fn reg(&mut self) -> &mut RegisterAllocator {
        &mut self.func_state.alloc
    }

    fn strings(&mut self) -> &mut Vec<String> {
        &mut self.func_state.strings
    }

    fn instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.func_state.instructions
    }

    fn push_instr(&mut self, instr: Instruction) {
        self.func_state.instructions.push(instr)
    }

    fn alloc_string(&mut self, str: String) -> StringID {
        let str_idx = self.strings().len();
        self.strings().push(str);
        StringID(str_idx.try_into().unwrap())
    }

    fn lookup_var(&mut self, ident: &str) -> VarLookup {
        let local_reg = self.func_state.scope_vars[..=(self.scope)]
            .into_iter()
            .rev()
            .find_map(|scope| scope.0.get(ident));

        if let Some(register) = local_reg {
            VarLookup::Local(*register)
        } else if let Some(register) = self.func_state.arguments.0.get(ident) {
            VarLookup::Argument(*register)
        } else {
            VarLookup::GlobalCell(self.func_state.global_values.cell_for_name(ident))
        }
    }

    fn define_local(&mut self, ident: String, location: LocalRegisterID) {
        self.func_state.scope_vars[self.scope]
            .0
            .insert(ident, location);
    }

    fn inner_scope(self) -> Self {
        if self.func_state.scope_vars.len() == self.scope - 1 {
            let scope = LocalScope::default();
            self.func_state.scope_vars.push(scope);
        } else {
            self.func_state.scope_vars[self.scope + 1].0.clear();
        }
        Self {
            scope: self.scope + 1,
            ..self
        }
    }

    fn new(func_state: &'a mut FunctionCompilationState<'b>) -> Self {
        if func_state.scope_vars.len() == 0 {
            let scope = LocalScope::default();
            func_state.scope_vars.push(scope);
        } else {
            func_state.scope_vars[0].0.clear();
        }
        Self {
            func_state,
            scope: 0,
        }
    }
}

pub fn compile_function(
    decl: &syn::FunctionDeclaration,
    global_values: &mut GlobalValues,
) -> (FnMeta, Vec<Instruction>) {
    use Instruction::*;
    let return_count = decl.body.ret.as_ref().map(|ret| ret.0.len()).unwrap_or(0);
    let mut state = FunctionCompilationState::with_args(decl.args.iter().cloned(), global_values);
    let mut root_scope = LocalFnCompState::new(&mut state);

    if let Some(syn::Return(exprs)) = &decl.body.ret {
        if let Some(expr) = exprs.first() {
            compile_expr(expr, &mut root_scope);
            root_scope.push_instr(StrRD(ArgumentRegisterID(0)));
        }
    }

    state.instructions.push(Ret);

    let meta = FnMeta {
        arg_count: decl.args.len().into(),
        const_strings: state.strings,
        label_mappings: vec![],
        return_count: MetaCount::Known(return_count),
        local_count: state.alloc.into_used_register_count(),
    };
    return (meta, state.instructions);
}

fn compile_expr(expr: &syn::Expression, state: &mut LocalFnCompState) {
    use Instruction::*;

    match expr {
        syn::Expression::Nil => {
            state.push_instr(ConstN);
        }
        syn::Expression::Number(num) if num.is_integer() => {
            state.push_instr(ConstI(num.as_i32()));
            state.push_instr(WrapI);
        }
        syn::Expression::Number(num) => {
            state.push_instr(ConstF(num.as_f64()));
            state.push_instr(WrapF);
        }
        syn::Expression::String(str) => {
            let str_id = state.alloc_string(str.0.clone());
            state.push_instr(ConstS(str_id));
            state.push_instr(WrapS);
        }
        syn::Expression::BinaryOperator { lhs, op, rhs } => {
            compile_binary_op(*op, lhs, rhs, state);
        }
        syn::Expression::Variable(syn::Var::Named(ident)) => {
            match state.lookup_var(ident.as_ref()) {
                VarLookup::Argument(reg) => state.push_instr(LdaRD(reg)),
                VarLookup::Local(reg) => state.push_instr(LdaLD(reg)),
                VarLookup::GlobalCell(cell) => state.push_instr(LdaDGl(cell)),
            };
        }
        _ => todo!(),
    }
}

fn compile_binary_op(
    op: syn::BinaryOperator,
    lhs: &syn::Expression,
    rhs: &syn::Expression,
    state: &mut LocalFnCompState,
) {
    use Instruction::*;

    compile_expr(lhs, state);
    let reg = state.reg().alloc_dyn();
    state.push_instr(StrLD(reg));
    compile_expr(rhs, state);

    let instr = match op {
        syn::BinaryOperator::Plus => DAddL,
        syn::BinaryOperator::Minus => DSubL,
        syn::BinaryOperator::Mul => DMulL,
        syn::BinaryOperator::Div => DDivL,
        _ => todo!(),
    };

    state.push_instr(instr(reg));
    state.reg().free_dyn();
}

#[cfg(test)]
mod test {
    use crate::reggie::{
        fn_meta::{LocalRegCount, MetaCount},
        ids::{ArgumentRegisterID, LocalRegisterID, StringID},
        machine::GlobalValues,
        ops::Instruction,
    };
    use crate::{error::LuaError, reggie::fn_meta::FnMeta, syn};

    use super::compile_function;

    use Instruction::*;

    macro_rules! test_instruction_output {
        ($name: ident, $code: expr, $instr: expr) => {
            #[test]
            fn $name() -> Result<(), LuaError> {
                let function = syn::lua_parser::function_declaration($code)?;
                let (meta, instructions) =
                    compile_function(&function, &mut GlobalValues::default());

                assert_eq!(
                    meta,
                    FnMeta {
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
        let (meta, instructions) = compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            FnMeta {
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
        let (meta, instructions) = compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            FnMeta {
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
        let (meta, instructions) = compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            FnMeta {
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
                let (meta, instructions) =
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
        let (meta, instructions) = compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            FnMeta {
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
        FnMeta {
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
        FnMeta {
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
        FnMeta {
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
        let (meta, instructions) = compile_function(&function, &mut GlobalValues::default());

        assert_eq!(
            meta,
            FnMeta {
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
