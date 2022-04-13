use crate::lang::{ArithmeticError, ArithmeticOperator, EvalError, LuaValue, TypeError};

use super::{
    compiler::CompiledModule,
    ids::{ArgumentRegisterID, BlockID, JmpLabel, LocalRegisterID, StringID},
    machine::{EqualityFlag, Machine, ProgramCounter, StackFrame, OrderingFlag},
    meta::MetaCount,
    ops::Instruction,
};

pub fn eval_loop(machine: &mut Machine) -> Result<(), EvalError> {
    let mut block = &machine.code_blocks[machine.program_counter.block];
    let mut position = machine.program_counter.position;
    let mut frame = machine
        .stack
        .last_mut()
        .expect("In order for VM to evaluate code, the stack should not be empty");
    loop {
        let instr = block.instructions[position as usize];
        match instr {
            Instruction::Ret => {
                block = &machine.code_blocks[frame.return_addr.block];
                position = frame.return_addr.position;
                machine.stack.pop();
                match machine.stack.last_mut() {
                    Some(new_frame) => frame = new_frame,
                    None => break,
                }
            }
            Instruction::ConstI(value) => {
                machine.accumulators.i = value;
                position += 1;
            }
            Instruction::WrapI => {
                machine.accumulators.d = LuaValue::number(machine.accumulators.i);
                position += 1;
            }
            Instruction::StrRD(reg) => {
                machine.argument_registers.d[reg.0 as usize] = machine.accumulators.d.clone();
                position += 1;
            }
            Instruction::LdaRD(reg) => {
                machine.accumulators.d = machine.argument_registers.d[reg.0 as usize].clone();
                position += 1;
            }
            Instruction::StrLD(reg) => {
                frame.local_values.d[reg.0 as usize] = machine.accumulators.d.clone();
                position += 1;
            }
            Instruction::LdaLD(reg) => {
                machine.accumulators.d = frame.local_values.d[reg.0 as usize].clone();
                position += 1;
            }
            Instruction::DAddR(reg) => {
                let res = binary_number_op(
                    &machine.accumulators.d,
                    &machine.argument_registers.d[reg.0 as usize],
                    ArithmeticOperator::Add,
                    std::ops::Add::add,
                )?;
                machine.accumulators.d = res;
                position += 1;
            }
            Instruction::DAddL(reg) => {
                let res = binary_number_op(
                    &machine.accumulators.d,
                    &frame.local_values.d[reg.0 as usize],
                    ArithmeticOperator::Add,
                    std::ops::Add::add,
                )?;
                machine.accumulators.d = res;
                position += 1;
            }
            Instruction::ConstN => {
                machine.accumulators.d = LuaValue::Nil;
                position += 1;
            }
            Instruction::ConstF(value) => {
                machine.accumulators.f = value;
                position += 1;
            }
            Instruction::WrapF => {
                machine.accumulators.d = LuaValue::number(machine.accumulators.f);
                position += 1;
            }
            Instruction::ConstS(StringID(string_id)) => {
                machine.accumulators.s = Some(block.meta.const_strings[string_id as usize].clone());
                position += 1;
            }
            Instruction::WrapS => {
                machine.accumulators.d =
                    LuaValue::String(machine.accumulators.s.as_ref().unwrap().clone());
                position += 1;
            }
            Instruction::ConstC(_) => todo!(),
            Instruction::WrapC => todo!(),
            Instruction::LdaDGl(cell_id) => {
                machine.accumulators.d = machine.global_values.value_of_cell(cell_id).clone();
                position += 1;
            }
            Instruction::EqTestRD(ArgumentRegisterID(reg)) => {
                machine.equality_flag = EqualityFlag::from_bool(
                    machine.accumulators.d == machine.argument_registers.d[reg as usize],
                );
                position += 1;
            }
            Instruction::EqTestLD(LocalRegisterID(reg)) => {
                machine.equality_flag = EqualityFlag::from_bool(
                    machine.accumulators.d == frame.local_values.d[reg as usize],
                );
                position += 1;
            }
            Instruction::Jmp(JmpLabel(jmp_label)) => {
                position = block.meta.label_mappings[jmp_label as usize]
                    .try_into()
                    .unwrap();
            }
            Instruction::Label => {
                /* nop */
                position += 1;
            }
            Instruction::JmpEQ(JmpLabel(jmp_label)) => {
                if machine.equality_flag == EqualityFlag::EQ {
                    position = block.meta.label_mappings[jmp_label as usize].try_into().unwrap();
                } else {
                    position += 1;
                }
            },
            Instruction::JmpNE(JmpLabel(jmp_label)) => {
                if machine.equality_flag == EqualityFlag::NE {
                    position = block.meta.label_mappings[jmp_label as usize].try_into().unwrap();
                } else {
                    position += 1;
                }
            },
            Instruction::JmpLT(JmpLabel(jmp_label)) => {
                if machine.equality_flag == EqualityFlag::NE && machine.ordering_flag == OrderingFlag::LT {
                    position = block.meta.label_mappings[jmp_label as usize].try_into().unwrap();
                } else {
                    position += 1;
                }
            },
            Instruction::JmpGT(JmpLabel(jmp_label)) => {
                if machine.equality_flag == EqualityFlag::NE && machine.ordering_flag == OrderingFlag::GT {
                    position = block.meta.label_mappings[jmp_label as usize].try_into().unwrap();
                } else {
                    position += 1;
                }
            },
            Instruction::JmpLE(JmpLabel(jmp_label)) => {
                if machine.ordering_flag == OrderingFlag::LT {
                    position = block.meta.label_mappings[jmp_label as usize].try_into().unwrap();
                } else {
                    position += 1;
                }
            },
            Instruction::JmpGE(JmpLabel(jmp_label)) => {
                if machine.ordering_flag == OrderingFlag::GT {
                    position = block.meta.label_mappings[jmp_label as usize].try_into().unwrap();
                } else {
                    position += 1;
                }
            },

            Instruction::LdaRF(_) => todo!(),
            Instruction::LdaRS(_) => todo!(),
            Instruction::LdaRI(_) => todo!(),
            Instruction::LdaRT(_) => todo!(),
            Instruction::LdaRC(_) => todo!(),
            Instruction::LdaRU(_) => todo!(),
            Instruction::LdaLF(_) => todo!(),
            Instruction::LdaLS(_) => todo!(),
            Instruction::LdaLI(_) => todo!(),
            Instruction::LdaLT(_) => todo!(),
            Instruction::LdaLC(_) => todo!(),
            Instruction::LdaLU(_) => todo!(),
            Instruction::StrRF(_) => todo!(),
            Instruction::StrRS(_) => todo!(),
            Instruction::StrRI(_) => todo!(),
            Instruction::StrRT(_) => todo!(),
            Instruction::StrRC(_) => todo!(),
            Instruction::StrRU(_) => todo!(),
            Instruction::StrLF(_) => todo!(),
            Instruction::StrLS(_) => todo!(),
            Instruction::StrLI(_) => todo!(),
            Instruction::StrLT(_) => todo!(),
            Instruction::StrLC(_) => todo!(),
            Instruction::StrLU(_) => todo!(),
            Instruction::LdaFGl(_) => todo!(),
            Instruction::LdaIGl(_) => todo!(),
            Instruction::LdaSGl(_) => todo!(),
            Instruction::LdaTGl(_) => todo!(),
            Instruction::LdaCGl(_) => todo!(),
            Instruction::LdaUGl(_) => todo!(),
            Instruction::StrFGl(_) => todo!(),
            Instruction::StrIGl(_) => todo!(),
            Instruction::StrSGl(_) => todo!(),
            Instruction::StrTGl(_) => todo!(),
            Instruction::StrCGl(_) => todo!(),
            Instruction::StrUGl(_) => todo!(),
            Instruction::StrDGl(_) => todo!(),
            Instruction::LdaDynGl => todo!(),
            Instruction::StrDynGl => todo!(),
            Instruction::FAddR(_) => todo!(),
            Instruction::FAddL(_) => todo!(),
            Instruction::FMulR(_) => todo!(),
            Instruction::FMulL(_) => todo!(),
            Instruction::FSubR(_) => todo!(),
            Instruction::FSubL(_) => todo!(),
            Instruction::FDivR(_) => todo!(),
            Instruction::FDivL(_) => todo!(),
            Instruction::IAddR(_) => todo!(),
            Instruction::IAddL(_) => todo!(),
            Instruction::IMulR(_) => todo!(),
            Instruction::IMulL(_) => todo!(),
            Instruction::ISubR(_) => todo!(),
            Instruction::ISubL(_) => todo!(),
            Instruction::IDivR(_) => todo!(),
            Instruction::IDivL(_) => todo!(),
            Instruction::DMulR(_) => todo!(),
            Instruction::DMulL(_) => todo!(),
            Instruction::DSubR(_) => todo!(),
            Instruction::DSubL(_) => todo!(),
            Instruction::DDivR(_) => todo!(),
            Instruction::DDivL(_) => todo!(),
            Instruction::SConcatR(_) => todo!(),
            Instruction::SConcatL(_) => todo!(),
            Instruction::DConcatR(_) => todo!(),
            Instruction::DConcatL(_) => todo!(),
            Instruction::IToS => todo!(),
            Instruction::FToS => todo!(),
            Instruction::DToS => todo!(),
            Instruction::SetVC => todo!(),
            Instruction::Call => todo!(),
            Instruction::TypedCall => todo!(),
            Instruction::DCall => todo!(),
            Instruction::EqTestRF(_) => todo!(),
            Instruction::EqTestRS(_) => todo!(),
            Instruction::EqTestRI(_) => todo!(),
            Instruction::EqTestRT(_) => todo!(),
            Instruction::EqTestRC(_) => todo!(),
            Instruction::EqTestRU(_) => todo!(),
            Instruction::EqTestLF(_) => todo!(),
            Instruction::EqTestLS(_) => todo!(),
            Instruction::EqTestLI(_) => todo!(),
            Instruction::EqTestLT(_) => todo!(),
            Instruction::EqTestLC(_) => todo!(),
            Instruction::EqTestLU(_) => todo!(),
            Instruction::TestRF(_) => todo!(),
            Instruction::TestRS(_) => todo!(),
            Instruction::TestRI(_) => todo!(),
            Instruction::TestRT(_) => todo!(),
            Instruction::TestRC(_) => todo!(),
            Instruction::TestRU(_) => todo!(),
            Instruction::TestRD(_) => todo!(),
            Instruction::TestLF(_) => todo!(),
            Instruction::TestLS(_) => todo!(),
            Instruction::TestLI(_) => todo!(),
            Instruction::TestLT(_) => todo!(),
            Instruction::TestLC(_) => todo!(),
            Instruction::TestLU(_) => todo!(),
            Instruction::TestLD(_) => todo!(),
            Instruction::TypeTest => todo!(),
            Instruction::NilTest => todo!(),
            Instruction::WrapT => todo!(),
            Instruction::WrapU => todo!(),
            Instruction::CastF => todo!(),
            Instruction::CastI => todo!(),
            Instruction::CastS => todo!(),
            Instruction::CastC => todo!(),
            Instruction::CastT => todo!(),
            Instruction::CastU => todo!(),
            Instruction::JmpN(_) => todo!(),
            Instruction::JmpF(_) => todo!(),
            Instruction::JmpI(_) => todo!(),
            Instruction::JmpC(_) => todo!(),
            Instruction::JmpT(_) => todo!(),
            Instruction::JmpU(_) => todo!(),
        }
    }

    Ok(())
}

fn binary_number_op(
    lhs: &LuaValue,
    rhs: &LuaValue,
    op: ArithmeticOperator,
    op_fn: impl FnOnce(f64, f64) -> f64,
) -> Result<LuaValue, TypeError> {
    if let (Some(lhs), Some(rhs)) = (lhs.as_number(), rhs.as_number()) {
        let res = op_fn(lhs.as_f64(), rhs.as_f64());
        Ok(LuaValue::number(res))
    } else {
        Err(TypeError::Arithmetic(ArithmeticError::Binary {
            lhs: lhs.clone(),
            rhs: rhs.clone(),
            op,
        }))
    }
}

pub fn call_block(machine: &mut Machine, block_id: BlockID) -> Result<&[LuaValue], EvalError> {
    let block = &machine.code_blocks[block_id];
    let return_count = block.meta.return_count;
    let stack_frame = StackFrame::new(
        &block.meta,
        ProgramCounter {
            block: BlockID(0),
            position: 0,
        },
    );
    machine.stack.push(stack_frame);
    machine.program_counter = ProgramCounter {
        block: block_id,
        position: 0,
    };
    eval_loop(machine)?;
    Ok(collect_dyn_returns(machine, return_count))
}

pub fn call_module(
    module: CompiledModule,
    machine: &mut Machine,
) -> Result<&[LuaValue], EvalError> {
    machine.code_blocks.extend(module.blocks);
    let top_level_block = machine.code_blocks.add(module.top_level);
    call_block(machine, top_level_block)
}

pub fn collect_dyn_returns(machine: &mut Machine, return_count: MetaCount) -> &[LuaValue] {
    let count = match return_count {
        MetaCount::Known(count) => count,
        MetaCount::Unknown => machine.value_count as usize,
    };

    &machine.argument_registers.d[..count]
}

#[cfg(test)]
mod test {
    use crate::lang::LuaValue;
    use crate::reggie::ids::{ArgumentRegisterID, JmpLabel, LocalRegisterID, StringID};
    use crate::reggie::machine::{
        CodeBlock, EqualityFlag, EqualityFlag::EQ, EqualityFlag::NE, Machine, OrderingFlag,
        OrderingFlag::GT, OrderingFlag::LT,
    };
    use crate::reggie::meta::{CodeMeta, LocalRegCount};
    use crate::reggie::ops::Instruction::{self, *};
    use crate::reggie::runtime::call_block;
    use ntest::timeout;

    macro_rules! test_instructions_with_meta {
        ($name: ident, [$($instr: expr),*$(,)?], $meta: expr, $post_condition: expr) => {
            #[test]
            #[timeout(5000)]
            fn $name() {
                let mut machine = Machine::new();
                let block_id = machine.code_blocks.add(CodeBlock {
                    meta: $meta,
                    instructions: vec![$($instr,)*],
                });
                call_block(&mut machine, block_id).unwrap();

                ($post_condition)(machine);
            }
        };
    }

    macro_rules! test_instructions_with_locals {
        ($name: ident, [$($instr: expr),*$(,)?], $locals: expr, $post_condition: expr) => {
            test_instructions_with_meta! {
                $name,
                [$($instr,)*],
                CodeMeta {
                    arg_count: 0.into(),
                    local_count: $locals,
                    return_count: 0.into(),
                    label_mappings: vec![],
                    const_strings: vec![],
                },
                $post_condition
            }
        };
    }

    macro_rules! test_instructions {
        ($name: ident, [$($instr: expr),*], $post_condition: expr) => {
            test_instructions_with_locals! {$name, [$($instr,)*], LocalRegCount::default(), $post_condition}
        };
    }

    macro_rules! test_instructions_with_strings {
        ($name: ident, [$($instr: expr),*], [$($strings: expr),*], $post_condition: expr) => {
            test_instructions_with_meta! {
                $name,
                [$($instr,)*],
                CodeMeta {
                    arg_count: 0.into(),
                    local_count: LocalRegCount::default(),
                    return_count: 0.into(),
                    label_mappings: vec![],
                    const_strings: vec![
                        $($strings.to_owned(),)*
                    ],
                },
                $post_condition
            }
        };
    }

    test_instructions!(ret_fn_call, [Ret], |_| {});

    test_instructions!(const_i, [ConstI(42), Ret], |machine: Machine| {
        assert_eq!(machine.accumulators.i, 42);
    });

    test_instructions!(wrap_i, [ConstI(42), WrapI, Ret], |machine: Machine| {
        assert_eq!(machine.accumulators.d, LuaValue::number(42));
    });

    test_instructions!(
        str_rd,
        [ConstI(42), WrapI, StrRD(ArgumentRegisterID(0)), Ret],
        |machine: Machine| { assert_eq!(machine.argument_registers.d[0], LuaValue::number(42)) }
    );

    test_instructions_with_locals!(
        str_and_lda_ld,
        [
            ConstI(42),
            WrapI,
            StrLD(LocalRegisterID(0)),
            ConstI(69),
            WrapI,
            LdaLD(LocalRegisterID(0)),
            Ret
        ],
        LocalRegCount {
            d: 1,
            ..Default::default()
        },
        |machine: Machine| { assert_eq!(machine.accumulators.d, LuaValue::number(42)) }
    );

    test_instructions!(
        str_and_lda_rd,
        [
            ConstI(42),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstI(69),
            WrapI,
            LdaRD(ArgumentRegisterID(0)),
            Ret
        ],
        |machine: Machine| { assert_eq!(machine.accumulators.d, LuaValue::number(42)) }
    );

    test_instructions_with_locals!(
        plus_1_and_2_local_regs,
        [
            ConstI(1),
            WrapI,
            StrLD(LocalRegisterID(0)),
            ConstI(2),
            WrapI,
            DAddL(LocalRegisterID(0)),
            StrRD(ArgumentRegisterID(0)),
            Ret,
        ],
        LocalRegCount {
            d: 1,
            ..Default::default()
        },
        |machine: Machine| {
            assert_eq!(machine.argument_registers.d[0], LuaValue::number(3));
        }
    );

    test_instructions_with_locals!(
        plus_1_and_2_arg_regs,
        [
            ConstI(1),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstI(2),
            WrapI,
            DAddR(ArgumentRegisterID(0)),
            StrRD(ArgumentRegisterID(0)),
            Ret,
        ],
        LocalRegCount {
            d: 1,
            ..Default::default()
        },
        |machine: Machine| {
            assert_eq!(machine.argument_registers.d[0], LuaValue::number(3));
        }
    );

    test_instructions!(
        const_n,
        [ConstI(42), WrapI, ConstN, Ret],
        |machine: Machine| { assert_eq!(machine.argument_registers.d[0], LuaValue::Nil) }
    );

    test_instructions!(const_f, [ConstF(42.4), Ret], |machine: Machine| {
        assert_eq!(machine.accumulators.f, 42.4)
    });

    test_instructions!(wrap_f, [ConstF(42.4), WrapF, Ret], |machine: Machine| {
        assert_eq!(machine.accumulators.d, LuaValue::number(42.4))
    });

    test_instructions_with_strings!(
        const_s,
        [ConstS(StringID(0)), Ret],
        ["hello"],
        |machine: Machine| { assert_eq!(machine.accumulators.s, Some("hello".to_owned())) }
    );

    test_instructions_with_strings!(
        wrap_s,
        [ConstS(StringID(0)), WrapS, Ret],
        ["hello"],
        |machine: Machine| { assert_eq!(machine.accumulators.d, LuaValue::string("hello")) }
    );

    #[quickcheck]
    #[timeout(5000)]
    fn lda_d_gl(value: LuaValue) {
        let mut machine = Machine::new();
        let cell_id = machine.global_values.set("value", value.clone());
        let block_id = machine.code_blocks.add(CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                local_count: LocalRegCount::default(),
                return_count: 0.into(),
                label_mappings: vec![],
                const_strings: vec![],
            },
            instructions: vec![LdaDGl(cell_id), Ret],
        });
        call_block(&mut machine, block_id).unwrap();
        assert!(machine.accumulators.d.total_eq(&value));
    }

    #[quickcheck]
    #[timeout(5000)]
    fn eq_test_d(lhs: LuaValue, rhs: LuaValue) {
        let mut machine = Machine::new();
        let expected = EqualityFlag::from_bool(lhs == rhs);
        machine.argument_registers.d[0] = lhs;
        machine.argument_registers.d[1] = rhs;
        let block_id = machine.code_blocks.add(CodeBlock {
            meta: CodeMeta {
                arg_count: 2.into(),
                local_count: LocalRegCount::default(),
                return_count: 0.into(),
                label_mappings: vec![],
                const_strings: vec![],
            },
            instructions: vec![
                LdaRD(ArgumentRegisterID(0)),
                EqTestRD(ArgumentRegisterID(1)),
                Ret,
            ],
        });
        call_block(&mut machine, block_id).unwrap();
        assert_eq!(machine.equality_flag, expected);
    }

    test_instructions_with_meta!(
        jmp,
        [ConstI(1), Jmp(JmpLabel(0)), ConstI(2), Label, Ret],
        CodeMeta {
            arg_count: 0.into(),
            const_strings: vec![],
            label_mappings: vec![3],
            local_count: LocalRegCount::default(),
            return_count: 0.into()
        },
        |machine: Machine| { assert_eq!(machine.accumulators.i, 1) }
    );

    static CONDITIONAL_JMP_BEHAVIOR: [(
        fn(JmpLabel) -> Instruction,
        Option<EqualityFlag>,
        Option<OrderingFlag>,
    ); 6] = [
        (JmpEQ, Some(EQ), None),
        (JmpNE, Some(NE), None),
        (JmpLT, Some(NE), Some(LT)),
        (JmpGT, Some(NE), Some(GT)),
        (JmpLE, None, Some(LT)),
        (JmpGE, None, Some(GT)),
    ];

    static FLAGS_PERMUTATION: [(EqualityFlag, OrderingFlag); 4] =
        [(EQ, LT), (EQ, GT), (NE, LT), (NE, GT)];

    #[test]
    fn conditional_jumps() {
        let mut machine = Machine::new();
        for (jmp_instr, triggered_eq_flag, triggered_ord_flag) in CONDITIONAL_JMP_BEHAVIOR {
            for (eq_flag, ord_flag) in FLAGS_PERMUTATION {
                let block_id = machine.code_blocks.add(CodeBlock {
                    meta: CodeMeta {
                        arg_count: 0.into(),
                        local_count: LocalRegCount::default(),
                        return_count: 0.into(),
                        label_mappings: vec![3],
                        const_strings: vec![],
                    },
                    instructions: vec![ConstI(1), jmp_instr(JmpLabel(0)), ConstI(2), Label, Ret],
                });
                machine.equality_flag = eq_flag;
                machine.ordering_flag = ord_flag;
                call_block(&mut machine, block_id).unwrap();
                let eq_flag_matches = triggered_eq_flag
                    .map(|flag| flag == eq_flag)
                    .unwrap_or(true);
                let ord_flag_matches = triggered_ord_flag
                    .map(|flag| flag == ord_flag)
                    .unwrap_or(true);
                let expected_value = if eq_flag_matches && ord_flag_matches {
                    1
                } else {
                    2
                };
                assert_eq!(machine.accumulators.i, expected_value);
            }
        }
    }
}
