use super::{
    ids::{ArgumentRegisterID, LocalRegisterID},
    machine::{EqualityFlag, Machine, OrderingFlag, ProgramCounter, StackFrame},
    ops::Instruction,
};
use crate::{
    ArithmeticError, EvalError, LuaKey, LuaValue, NativeFunction, NativeFunctionKind, TableRef,
    TableValue, TypeError,
};
use luar_error::ArithmeticOperator;
use luar_lex::Ident;
use std::borrow::Borrow;

pub fn eval_loop(machine: &mut Machine) -> Result<(), EvalError> {
    let mut block = &machine.code_blocks[machine.program_counter.block];
    let mut position = &mut machine.program_counter.position;
    let mut frame = machine
        .stack
        .last_mut()
        .expect("In order for VM to evaluate code, the stack should not be empty");
    loop {
        let instr = block.instructions[*position as usize];
        match instr {
            Instruction::Ret => {
                block = &machine.code_blocks[frame.return_addr.block];
                machine.program_counter = frame.return_addr;
                position = &mut machine.program_counter.position;
                machine.stack.pop();
                match machine.stack.last_mut() {
                    Some(new_frame) => frame = new_frame,
                    None => return Ok(()),
                }
            }
            Instruction::ConstI(value) => {
                machine.accumulators.i = value;
                *position += 1;
            }
            Instruction::WrapI => {
                machine.accumulators.d = LuaValue::Int(machine.accumulators.i);
                *position += 1;
            }
            Instruction::StrRD(reg) => {
                machine.argument_registers.d[reg.0 as usize] = machine.accumulators.d.clone();
                *position += 1;
            }
            Instruction::LdaRD(reg) => {
                machine.accumulators.d = machine.argument_registers.d[reg.0 as usize].clone();
                *position += 1;
            }
            Instruction::StrLD(reg) => {
                frame.local_values.d[reg.0 as usize] = machine.accumulators.d.clone();
                *position += 1;
            }
            Instruction::LdaLD(reg) => {
                machine.accumulators.d = frame.local_values.d[reg.0 as usize].clone();
                *position += 1;
            }
            Instruction::DAddR(reg) => {
                machine.accumulators.d = add_dyn(
                    &machine.accumulators.d,
                    &machine.argument_registers.d[reg.0 as usize],
                )?;
                *position += 1;
            }
            Instruction::DAddL(reg) => {
                machine.accumulators.d = add_dyn(
                    &machine.accumulators.d,
                    &frame.local_values.d[reg.0 as usize],
                )?;
                *position += 1;
            }
            Instruction::ConstN => {
                machine.accumulators.d = LuaValue::Nil;
                *position += 1;
            }
            Instruction::ConstF(value) => {
                machine.accumulators.f = value;
                *position += 1;
            }
            Instruction::WrapF => {
                machine.accumulators.d = LuaValue::Float(machine.accumulators.f);
                *position += 1;
            }
            Instruction::ConstS(string_id) => {
                machine.accumulators.s = Some(block.meta.const_strings[string_id].clone());
                *position += 1;
            }
            Instruction::WrapS => {
                machine.accumulators.d =
                    LuaValue::String(machine.accumulators.s.as_ref().unwrap().clone());
                *position += 1;
            }
            Instruction::ConstC(local_block_id) => {
                machine.accumulators.c =
                    machine.code_blocks.blocks_of_module(block.module)[local_block_id];
                *position += 1;
            }
            Instruction::WrapC => {
                machine.accumulators.d = LuaValue::Function(machine.accumulators.c);
                *position += 1;
            }
            Instruction::LdaDGl(cell_id) => {
                machine.accumulators.d = machine.global_values.value_of_cell(cell_id).clone();
                *position += 1;
            }
            Instruction::EqTestRD(ArgumentRegisterID(reg)) => {
                machine.equality_flag = EqualityFlag::from_bool(
                    machine.accumulators.d == machine.argument_registers.d[reg as usize],
                );
                *position += 1;
            }
            Instruction::EqTestLD(LocalRegisterID(reg)) => {
                machine.equality_flag = EqualityFlag::from_bool(
                    machine.accumulators.d == frame.local_values.d[reg as usize],
                );
                *position += 1;
            }
            Instruction::Jmp(jmp_label) => {
                *position = block.meta.label_mappings[jmp_label];
            }
            Instruction::Label => {
                /* nop */
                *position += 1;
            }
            Instruction::JmpEQ(jmp_label) => {
                if machine.equality_flag == EqualityFlag::EQ {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpNE(jmp_label) => {
                if machine.equality_flag == EqualityFlag::NE {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpLT(jmp_label) => {
                if machine.equality_flag == EqualityFlag::NE
                    && machine.ordering_flag == OrderingFlag::LT
                {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpGT(jmp_label) => {
                if machine.equality_flag == EqualityFlag::NE
                    && machine.ordering_flag == OrderingFlag::GT
                {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpLE(jmp_label) => {
                if machine.ordering_flag == OrderingFlag::LT {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpGE(jmp_label) => {
                if machine.ordering_flag == OrderingFlag::GT {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::StrDGl(cell) => {
                machine
                    .global_values
                    .set_cell(cell, machine.accumulators.d.clone());
                *position += 1;
            }
            Instruction::StrVC => {
                machine.value_count = machine.accumulators.i.try_into().unwrap();
                *position += 1;
            }
            Instruction::DCall => match machine.accumulators.d.clone() {
                LuaValue::Function(block_id) => {
                    let new_block = &machine.code_blocks[block_id];
                    let new_frame = StackFrame::new(
                        &new_block.meta,
                        ProgramCounter {
                            position: *position + 1,
                            block: machine.program_counter.block,
                        },
                    );
                    machine.stack.push(new_frame);
                    frame = machine
                        .stack
                        .last_mut()
                        .expect("New stack frame have just been pushed");
                    block = new_block;
                    *position = 0;
                    machine.program_counter.block = block_id;
                }
                LuaValue::NativeFunction(NativeFunction(native_fn_kind)) => {
                    match native_fn_kind.borrow() {
                        NativeFunctionKind::Dyn(dyn_fn) => {
                            dyn_fn.call(&mut machine.argument_registers, machine.value_count)?;
                            machine.value_count = dyn_fn.return_count();
                        }
                        NativeFunctionKind::OverloadSet(_) => {
                            todo!("Cannot call native functions defined with overload sets yet")
                        }
                    };
                    *position += 1;
                }
                _ => {
                    return Err(EvalError::from(TypeError::IsNotCallable(
                        machine.accumulators.d.clone(),
                    )))
                }
            },
            Instruction::LdaProt(reg) => {
                machine.accumulators.d = if machine.value_count > reg.0 {
                    machine.argument_registers.d[reg.0 as usize].clone()
                } else {
                    LuaValue::Nil
                };
                *position += 1;
            }
            Instruction::TypedCall => {
                let new_block = &machine.code_blocks[machine.accumulators.c];
                let new_frame = StackFrame::new(
                    &new_block.meta,
                    ProgramCounter {
                        position: *position + 1,
                        block: machine.program_counter.block,
                    },
                );
                machine.stack.push(new_frame);
                frame = machine
                    .stack
                    .last_mut()
                    .expect("New stack frame have just been pushed");
                block = new_block;
                *position = 0;
                machine.program_counter.block = machine.accumulators.c;
            }
            Instruction::RDShiftRight => {
                machine
                    .argument_registers
                    .d
                    .rotate_right((machine.accumulators.i as u16) as usize);
                *position += 1;
            }
            Instruction::LdaVC => {
                machine.accumulators.i = machine.value_count as i32;
                *position += 1;
            }
            Instruction::IAddR(reg) => {
                machine.accumulators.i += machine.argument_registers.i[reg.0 as usize];
                *position += 1;
            }
            Instruction::IAddL(reg) => {
                machine.accumulators.i += frame.local_values.i[reg.0 as usize];
                *position += 1;
            }
            Instruction::StrLI(reg) => {
                frame.local_values.i[reg.0 as usize] = machine.accumulators.i;
                *position += 1;
            }
            Instruction::LdaLI(reg) => {
                machine.accumulators.i = frame.local_values.i[reg.0 as usize];
                *position += 1;
            }
            Instruction::StrRI(reg) => {
                machine.argument_registers.i[reg.0 as usize] = machine.accumulators.i;
                *position += 1;
            }
            Instruction::LdaRI(reg) => {
                machine.accumulators.i = machine.argument_registers.i[reg.0 as usize];
                *position += 1;
            }
            Instruction::NilTest => {
                machine.equality_flag =
                    EqualityFlag::from_bool(machine.accumulators.d == LuaValue::Nil);
                *position += 1;
            }
            Instruction::DSubR(reg) => {
                machine.accumulators.d = sub_dyn(
                    &machine.accumulators.d,
                    &machine.argument_registers.d[reg.0 as usize],
                )?;
                *position += 1;
            }
            Instruction::DSubL(reg) => {
                machine.accumulators.d = sub_dyn(
                    &machine.accumulators.d,
                    &frame.local_values.d[reg.0 as usize],
                )?;
                *position += 1;
            }
            Instruction::NewT => {
                machine.accumulators.t = Some(TableRef::from(TableValue::new()));
                *position += 1;
            }
            Instruction::StrRT(reg) => {
                machine.argument_registers.t[reg.0 as usize] = machine.accumulators.t.clone();
                *position += 1;
            }
            Instruction::LdaRT(reg) => {
                machine.accumulators.t = machine.argument_registers.t[reg.0 as usize].clone();
                *position += 1;
            }
            Instruction::LdaLT(reg) => {
                machine.accumulators.t = frame.local_values.t[reg.0 as usize].clone();
                *position += 1;
            }
            Instruction::StrLT(reg) => {
                frame.local_values.t[reg.0 as usize] = machine.accumulators.t.clone();
                *position += 1;
            }
            Instruction::PushD => {
                let table = machine.accumulators.t.as_mut().unwrap();
                table.push(machine.accumulators.d.clone());
                *position += 1;
            }
            Instruction::AssocASD => {
                let table = machine.accumulators.t.as_mut().unwrap();
                table.assoc_str(
                    machine.accumulators.s.as_ref().unwrap(),
                    machine.accumulators.d.clone(),
                );
                *position += 1;
            }
            Instruction::CastT => {
                machine.equality_flag = if let LuaValue::Table(ref table) = machine.accumulators.d {
                    machine.accumulators.t = Some(table.clone());
                    EqualityFlag::EQ
                } else {
                    EqualityFlag::NE
                };
                *position += 1;
            }
            Instruction::TablePropertyLookupError => {
                return Err(EvalError::from(TypeError::CannotAccessProperty {
                    property: Ident::new(machine.accumulators.s.take().unwrap()),
                    of: std::mem::replace(&mut machine.accumulators.d, LuaValue::Nil),
                }))
            }
            Instruction::TableMemberLookupErrorR(reg) => {
                return Err(EvalError::from(TypeError::CannotAccessMember {
                    member: std::mem::replace(
                        &mut machine.argument_registers.d[reg.0 as usize],
                        LuaValue::Nil,
                    ),
                    of: std::mem::replace(&mut machine.accumulators.d, LuaValue::Nil),
                }))
            }
            Instruction::TableMemberLookupErrorL(reg) => {
                return Err(EvalError::from(TypeError::CannotAccessMember {
                    member: std::mem::replace(
                        &mut frame.local_values.d[reg.0 as usize],
                        LuaValue::Nil,
                    ),
                    of: std::mem::replace(&mut machine.accumulators.d, LuaValue::Nil),
                }))
            }
            Instruction::WrapT => {
                machine.accumulators.d =
                    LuaValue::Table(machine.accumulators.t.as_ref().unwrap().clone());
                *position += 1;
            }
            Instruction::LdaAssocAS => {
                machine.accumulators.d = machine
                    .accumulators
                    .t
                    .as_mut()
                    .unwrap()
                    .get_str_assoc(machine.accumulators.s.as_ref().unwrap());
                *position += 1;
            }
            Instruction::LdaAssocAD => {
                match LuaKey::try_from(machine.accumulators.d.clone()) {
                    Ok(key) => {
                        machine.accumulators.d = machine.accumulators.t.as_mut().unwrap().get(&key);
                    },
                    Err(_) => {
                        machine.accumulators.d = LuaValue::Nil;
                    }
                }
                *position += 1;
            }
            Instruction::DMulR(reg) => {
                machine.accumulators.d = mul_dyn(
                    &machine.accumulators.d,
                    &machine.argument_registers.d[reg.0 as usize],
                )?;
                *position += 1;
            }
            Instruction::DMulL(reg) => {
                machine.accumulators.d = mul_dyn(
                    &machine.accumulators.d,
                    &frame.local_values.d[reg.0 as usize],
                )?;
                *position += 1;
            }
            Instruction::DDivR(reg) => {
                machine.accumulators.d = div_dyn(
                    &machine.accumulators.d,
                    &machine.argument_registers.d[reg.0 as usize],
                )?;
                *position += 1;
            }
            Instruction::DDivL(reg) => {
                machine.accumulators.d = div_dyn(
                    &machine.accumulators.d,
                    &frame.local_values.d[reg.0 as usize],
                )?;
                *position += 1;
            }

            Instruction::LdaRF(_) => todo!(),
            Instruction::LdaRS(_) => todo!(),
            Instruction::LdaRC(_) => todo!(),
            Instruction::LdaRU(_) => todo!(),
            Instruction::LdaLF(_) => todo!(),
            Instruction::LdaLS(_) => todo!(),
            Instruction::LdaLC(_) => todo!(),
            Instruction::LdaLU(_) => todo!(),
            Instruction::StrRF(_) => todo!(),
            Instruction::StrRS(_) => todo!(),
            Instruction::StrRC(_) => todo!(),
            Instruction::StrRU(_) => todo!(),
            Instruction::StrLF(_) => todo!(),
            Instruction::StrLS(_) => todo!(),
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
            Instruction::IMulR(_) => todo!(),
            Instruction::IMulL(_) => todo!(),
            Instruction::ISubR(_) => todo!(),
            Instruction::ISubL(_) => todo!(),
            Instruction::IDivR(_) => todo!(),
            Instruction::IDivL(_) => todo!(),
            Instruction::SConcatR(_) => todo!(),
            Instruction::SConcatL(_) => todo!(),
            Instruction::DConcatR(_) => todo!(),
            Instruction::DConcatL(_) => todo!(),
            Instruction::IToS => todo!(),
            Instruction::FToS => todo!(),
            Instruction::DToS => todo!(),
            Instruction::Call => todo!(),
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
            Instruction::WrapU => todo!(),
            Instruction::CastF => todo!(),
            Instruction::CastI => todo!(),
            Instruction::CastS => todo!(),
            Instruction::CastC => todo!(),
            Instruction::CastU => todo!(),
            Instruction::JmpN(_) => todo!(),
            Instruction::JmpF(_) => todo!(),
            Instruction::JmpI(_) => todo!(),
            Instruction::JmpC(_) => todo!(),
            Instruction::JmpT(_) => todo!(),
            Instruction::JmpU(_) => todo!(),
            Instruction::RFShiftRight => todo!(),
            Instruction::RIShiftRight => todo!(),
            Instruction::RSShiftRight => todo!(),
            Instruction::RTShiftRight => todo!(),
            Instruction::RCShiftRight => todo!(),
            Instruction::RUShiftRight => todo!(),
            Instruction::AssocRD(_) => todo!(),
            Instruction::AssocLD(_) => todo!(),
        }
    }
}

fn sub_dyn(lhs: &LuaValue, rhs: &LuaValue) -> Result<LuaValue, TypeError> {
    match (lhs, rhs) {
        (LuaValue::Int(lhs), LuaValue::Int(rhs)) => Ok(LuaValue::Int(lhs - rhs)),
        (lhs, rhs) => {
            if let (Some(lhs), Some(rhs)) = (lhs.coerce_to_f64(), rhs.coerce_to_f64()) {
                Ok(LuaValue::Float(lhs - rhs))
            } else {
                Err(TypeError::Arithmetic(ArithmeticError::Binary {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    op: ArithmeticOperator::Sub,
                }))
            }
        }
    }
}

fn add_dyn(lhs: &LuaValue, rhs: &LuaValue) -> Result<LuaValue, TypeError> {
    match (lhs, rhs) {
        (LuaValue::Int(lhs), LuaValue::Int(rhs)) => Ok(LuaValue::Int(lhs + rhs)),
        (lhs, rhs) => {
            if let (Some(lhs), Some(rhs)) = (lhs.coerce_to_f64(), rhs.coerce_to_f64()) {
                Ok(LuaValue::Float(lhs + rhs))
            } else {
                Err(TypeError::Arithmetic(ArithmeticError::Binary {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    op: ArithmeticOperator::Add,
                }))
            }
        }
    }
}

fn div_dyn(lhs: &LuaValue, rhs: &LuaValue) -> Result<LuaValue, TypeError> {
    if let (Some(lhs), Some(rhs)) = (lhs.coerce_to_f64(), rhs.coerce_to_f64()) {
        Ok(LuaValue::Float(lhs / rhs))
    } else {
        Err(TypeError::Arithmetic(ArithmeticError::Binary {
            lhs: lhs.clone(),
            rhs: rhs.clone(),
            op: ArithmeticOperator::Div,
        }))
    }
}

fn mul_dyn(lhs: &LuaValue, rhs: &LuaValue) -> Result<LuaValue, TypeError> {
    match (lhs, rhs) {
        (LuaValue::Int(lhs), LuaValue::Int(rhs)) => Ok(LuaValue::Int(lhs * rhs)),
        (lhs, rhs) => {
            if let (Some(lhs), Some(rhs)) = (lhs.coerce_to_f64(), rhs.coerce_to_f64()) {
                Ok(LuaValue::Float(lhs * rhs))
            } else {
                Err(TypeError::Arithmetic(ArithmeticError::Binary {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    op: ArithmeticOperator::Mul,
                }))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        call_block,
        compiler::CompiledModule,
        ids::{ArgumentRegisterID, JmpLabel, LocalBlockID, LocalRegisterID, StringID},
        keyed_vec::keyed_vec,
        machine::{
            CodeBlock, EqualityFlag, EqualityFlag::EQ, EqualityFlag::NE, Machine, OrderingFlag,
            OrderingFlag::GT, OrderingFlag::LT,
        },
        meta::{reg_count, CodeMeta, LocalRegCount},
        ops::Instruction::{self, *},
        EvalError, LuaValue, NativeFunction, Strict,
    };
    use ntest::timeout;

    macro_rules! test_instructions_with_meta {
        ($name: ident, [$($instr: expr),*$(,)?], $meta: expr, $post_condition: expr) => {
            #[test]
            #[timeout(5000)]
            fn $name() {
                let mut machine = Machine::new();
                let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
                    meta: $meta,
                    instructions: vec![$($instr,)*],
                });
                call_block::<()>(block_id, &mut machine).unwrap();

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
                    ..Default::default()
                },
                $post_condition
            }
        };
    }

    macro_rules! test_instructions {
        ($name: ident, [$($instr: expr),*$(,)?], $post_condition: expr) => {
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
                    return_count: 0.into(),
                    const_strings: $crate::keyed_vec::keyed_vec![
                        $($strings.to_owned(),)*
                    ],
                    ..Default::default()
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
        assert_eq!(machine.accumulators.d, LuaValue::Int(42));
    });

    test_instructions!(
        str_rd,
        [ConstI(42), WrapI, StrRD(ArgumentRegisterID(0)), Ret],
        |machine: Machine| { assert_eq!(machine.argument_registers.d[0], LuaValue::Int(42)) }
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
        reg_count! { D: 1 },
        |machine: Machine| { assert_eq!(machine.accumulators.d, LuaValue::Int(42)) }
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
        |machine: Machine| { assert_eq!(machine.accumulators.d, LuaValue::Int(42)) }
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
        reg_count! { D: 1 },
        |machine: Machine| {
            assert_eq!(machine.argument_registers.d[0].coerce_to_f64(), Some(3.0));
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
        reg_count! { D: 1 },
        |machine: Machine| {
            assert_eq!(machine.argument_registers.d[0].coerce_to_f64(), Some(3.0));
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
        assert_eq!(machine.accumulators.d, LuaValue::Float(42.4))
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

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    // #[timeout(5000)]
    fn lda_d_gl(value: LuaValue) {
        let mut machine = Machine::new();
        let cell_id = machine.global_values.set("value", value.clone());
        let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                return_count: 0.into(),
                ..Default::default()
            },
            instructions: vec![LdaDGl(cell_id), Ret],
        });
        call_block::<Strict<()>>(block_id, &mut machine).unwrap();
        assert!(machine.accumulators.d.total_eq(&value));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    // #[timeout(5000)]
    fn eq_test_d(lhs: LuaValue, rhs: LuaValue) {
        let mut machine = Machine::new();
        let expected = EqualityFlag::from_bool(lhs == rhs);
        machine.argument_registers.d[0] = lhs;
        machine.argument_registers.d[1] = rhs;
        let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
            meta: CodeMeta {
                arg_count: 2.into(),
                return_count: 0.into(),
                ..Default::default()
            },
            instructions: vec![
                LdaRD(ArgumentRegisterID(0)),
                EqTestRD(ArgumentRegisterID(1)),
                Ret,
            ],
        });
        call_block::<Strict<()>>(block_id, &mut machine).unwrap();
        assert_eq!(machine.equality_flag, expected);
    }

    test_instructions_with_meta!(
        jmp,
        [ConstI(1), Jmp(JmpLabel(0)), ConstI(2), Label, Ret],
        CodeMeta {
            arg_count: 0.into(),
            label_mappings: keyed_vec![3],
            return_count: 0.into(),
            ..Default::default()
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
                let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
                    meta: CodeMeta {
                        arg_count: 0.into(),
                        return_count: 0.into(),
                        label_mappings: keyed_vec![3],
                        ..Default::default()
                    },
                    instructions: vec![ConstI(1), jmp_instr(JmpLabel(0)), ConstI(2), Label, Ret],
                });
                machine.equality_flag = eq_flag;
                machine.ordering_flag = ord_flag;
                call_block::<Strict<()>>(block_id, &mut machine).unwrap();
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

    #[test]
    #[timeout(5000)]
    fn str_d_gl() {
        let mut machine = Machine::new();

        let cell = machine.global_values.cell_for_name("global_value");
        assert_eq!(machine.global_values.value_of_cell(cell), &LuaValue::Nil);

        let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                return_count: 0.into(),
                ..Default::default()
            },
            instructions: vec![ConstI(42), WrapI, StrDGl(cell), Ret],
        });
        call_block::<()>(block_id, &mut machine).unwrap();

        assert_eq!(
            machine.global_values.value_of_cell(cell),
            &LuaValue::Int(42)
        );
        assert_eq!(
            machine.global_values.get("global_value"),
            &LuaValue::Int(42)
        )
    }

    test_instructions!(set_vc, [ConstI(42), StrVC, Ret], |machine: Machine| {
        assert_eq!(machine.value_count, 42);
    });

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    fn d_call_on_uncallable(value: LuaValue) -> quickcheck::TestResult {
        if value.is_function() {
            return quickcheck::TestResult::discard();
        }

        let mut machine = Machine::new();

        let value_cell = machine.global_values.set("not_a_function", value);

        let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                return_count: 0.into(),
                ..Default::default()
            },
            instructions: vec![LdaDGl(value_cell), DCall, Ret],
        });
        let res = call_block::<()>(block_id, &mut machine);

        luar_error::assert_type_error!(luar_error::TypeError::IsNotCallable(_), res);
        quickcheck::TestResult::passed()
    }

    #[test]
    fn d_call_native_function() {
        let mut machine = Machine::new();

        let function = NativeFunction::new(|num| match num {
            LuaValue::Int(int) => LuaValue::Int(int + 1),
            _ => panic!("Unexpected value {}", num),
        });
        let value_cell = machine
            .global_values
            .set("not_a_function", LuaValue::NativeFunction(function));

        let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                return_count: 1.into(),
                ..Default::default()
            },
            instructions: vec![
                ConstI(68),
                WrapI,
                StrRD(ArgumentRegisterID(0)),
                LdaDGl(value_cell),
                ConstI(1),
                StrVC,
                DCall,
                Ret,
            ],
        });
        let res = call_block::<LuaValue>(block_id, &mut machine).unwrap();
        assert_eq!(res, LuaValue::Int(69));
    }

    #[test]
    fn d_call_native_function_propagates_errors() {
        let mut machine = Machine::new();

        let function =
            NativeFunction::new(|| Result::<(), EvalError>::Err(EvalError::AssertionError));
        let value_cell = machine
            .global_values
            .set("not_a_function", LuaValue::NativeFunction(function));

        let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                return_count: 0.into(),
                ..Default::default()
            },
            instructions: vec![LdaDGl(value_cell), ConstI(1), StrVC, DCall, Ret],
        });
        let res = call_block::<()>(block_id, &mut machine);
        assert!(matches!(res, Err(EvalError::AssertionError)));
    }

    #[test]
    fn d_call_lua_function() {
        let mut machine = Machine::new();

        let module = CompiledModule {
            blocks: keyed_vec![CodeBlock {
                meta: CodeMeta {
                    arg_count: 1.into(),
                    return_count: 1.into(),
                    ..Default::default()
                },
                instructions: vec![
                    ConstI(1),
                    WrapI,
                    DAddR(ArgumentRegisterID(0)),
                    StrRD(ArgumentRegisterID(0)),
                    Ret,
                ],
            }],
            top_level: CodeBlock {
                meta: CodeMeta {
                    arg_count: 0.into(),
                    return_count: 1.into(),
                    ..Default::default()
                },
                instructions: vec![
                    ConstI(68),
                    WrapI,
                    StrRD(ArgumentRegisterID(0)),
                    ConstI(1),
                    StrVC,
                    ConstC(LocalBlockID(0)),
                    WrapC,
                    DCall,
                    Ret,
                ],
            },
        };

        let top_level_block = machine.code_blocks.add_module(module);

        let res = call_block::<LuaValue>(top_level_block, &mut machine).unwrap();
        assert_eq!(res, LuaValue::Int(69));
    }

    #[test]
    fn const_c_and_wrap_c() {
        let mut machine = Machine::new();

        let block1 = CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                return_count: 1.into(),
                ..Default::default()
            },
            instructions: vec![ConstI(42), WrapI, StrRD(ArgumentRegisterID(0)), Ret],
        };
        let block2 = CodeBlock {
            meta: CodeMeta {
                arg_count: 0.into(),
                return_count: 1.into(),
                ..Default::default()
            },
            instructions: vec![ConstI(69), WrapI, StrRD(ArgumentRegisterID(0)), Ret],
        };

        let module = CompiledModule {
            blocks: keyed_vec![block1.clone(), block2.clone()],
            top_level: CodeBlock {
                meta: CodeMeta {
                    ..Default::default()
                },
                instructions: vec![ConstC(LocalBlockID(0)), WrapC, ConstC(LocalBlockID(1)), Ret],
            },
        };
        let top_level_block = machine.code_blocks.add_module(module);
        call_block::<()>(top_level_block, &mut machine).unwrap();

        let block1_id = machine.accumulators.d.unwrap_lua_function();
        let block2_id = machine.accumulators.c;

        assert_eq!(
            machine.code_blocks[block1_id].instructions,
            block1.instructions
        );
        assert_eq!(
            machine.code_blocks[block2_id].instructions,
            block2.instructions
        );
    }

    test_instructions!(
        lda_prot,
        [
            ConstI(42),
            WrapI,
            StrRD(ArgumentRegisterID(2)),
            ConstI(69),
            WrapI,
            StrRD(ArgumentRegisterID(3)),
            ConstI(3),
            StrVC,
            LdaProt(ArgumentRegisterID(2)),
            StrRD(ArgumentRegisterID(0)),
            LdaProt(ArgumentRegisterID(3)),
            StrRD(ArgumentRegisterID(1)),
            Ret
        ],
        |machine: Machine| {
            assert_eq!(machine.argument_registers.d[0], LuaValue::Int(42));
            assert_eq!(machine.argument_registers.d[1], LuaValue::Nil);
        }
    );

    #[test]
    fn typed_call() {
        let mut machine = Machine::new();

        let module = CompiledModule {
            blocks: keyed_vec![CodeBlock {
                meta: CodeMeta {
                    arg_count: 1.into(),
                    return_count: 1.into(),
                    ..Default::default()
                },
                instructions: vec![
                    ConstI(1),
                    WrapI,
                    DAddR(ArgumentRegisterID(0)),
                    StrRD(ArgumentRegisterID(0)),
                    Ret,
                ],
            }],
            top_level: CodeBlock {
                meta: CodeMeta {
                    arg_count: 0.into(),
                    return_count: 1.into(),
                    ..Default::default()
                },
                instructions: vec![
                    ConstI(68),
                    WrapI,
                    StrRD(ArgumentRegisterID(0)),
                    ConstI(1),
                    StrVC,
                    ConstC(LocalBlockID(0)),
                    TypedCall,
                    Ret,
                ],
            },
        };

        let top_level_block = machine.code_blocks.add_module(module);

        let res = call_block::<LuaValue>(top_level_block, &mut machine).unwrap();
        assert_eq!(res, LuaValue::Int(69));
    }

    test_instructions!(
        rd_shift_right,
        [
            ConstI(1),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstI(2),
            WrapI,
            StrRD(ArgumentRegisterID(1)),
            ConstI(3),
            WrapI,
            StrRD(ArgumentRegisterID(2)),
            ConstI(2),
            RDShiftRight,
            Ret
        ],
        |machine: Machine| {
            assert_eq!(machine.argument_registers.d[2], LuaValue::Int(1));
            assert_eq!(machine.argument_registers.d[3], LuaValue::Int(2));
            assert_eq!(machine.argument_registers.d[4], LuaValue::Int(3));
        }
    );

    test_instructions!(
        lda_vc,
        [ConstI(69), StrVC, ConstI(42), LdaVC, Ret],
        |machine: Machine| {
            assert_eq!(machine.accumulators.i, 69);
        }
    );

    test_instructions_with_locals!(
        str_li_and_lda_li,
        [
            ConstI(69),
            StrLI(LocalRegisterID(0)),
            ConstI(42),
            LdaLI(LocalRegisterID(0)),
            Ret
        ],
        reg_count! { I: 1 },
        |machine: Machine| {
            assert_eq!(machine.accumulators.i, 69);
        }
    );

    test_instructions!(
        str_ri_and_lda_ri,
        [
            ConstI(69),
            StrRI(ArgumentRegisterID(0)),
            ConstI(42),
            LdaRI(ArgumentRegisterID(0)),
            Ret
        ],
        |machine: Machine| {
            assert_eq!(machine.accumulators.i, 69);
        }
    );

    test_instructions_with_locals!(
        i_add,
        [
            ConstI(68000),
            StrLI(LocalRegisterID(0)),
            ConstI(420),
            StrRI(ArgumentRegisterID(0)),
            ConstI(1000),
            IAddL(LocalRegisterID(0)),
            IAddR(ArgumentRegisterID(0)),
            Ret
        ],
        reg_count! { I: 1 },
        |machine: Machine| {
            assert_eq!(machine.accumulators.i, 69420);
        }
    );

    test_instructions!(
        nil_test_nil,
        [
            ConstI(1),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstI(2),
            EqTestRD(ArgumentRegisterID(0)),
            ConstN,
            NilTest,
            Ret
        ],
        |machine: Machine| {
            assert_eq!(machine.equality_flag, EqualityFlag::EQ);
        }
    );

    test_instructions!(
        nil_test_non_nill,
        [
            ConstI(1),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstI(1),
            EqTestRD(ArgumentRegisterID(0)),
            NilTest,
            Ret
        ],
        |machine: Machine| {
            assert_eq!(machine.equality_flag, EqualityFlag::NE);
        }
    );

    // D arithmetic functions are such a pain to spec and write test cases for.
    // TODO: write arithmetic tests

    // test_instructions_with_locals!(
    //     d_sub,
    //     [
    //         ConstI(228),
    //         WrapI,
    //         StrLD(LocalRegisterID(0)),
    //         ConstI(-27),
    //         WrapI,
    //         StrRD(ArgumentRegisterID(0)),
    //         ConstI(186),
    //         WrapI,
    //         DSubL(LocalRegisterID(0)),
    //         DSubR(ArgumentRegisterID(0)),
    //         Ret
    //     ],
    //     LocalRegCount {
    //         d: 1,
    //         ..Default::default()
    //     },
    //     |machine: Machine| {
    //         assert_eq!(machine.accumulators.d, LuaValue::Int(69));
    //     }
    // );

    test_instructions!(
        new_and_str_rt,
        [NewT, StrRT(ArgumentRegisterID(0)), NewT, Ret],
        |machine: Machine| {
            assert!(machine.accumulators.t.is_some());
            assert!(machine.argument_registers.t[0].is_some());
            assert_ne!(machine.argument_registers.t[0], machine.accumulators.t);
        }
    );

    test_instructions!(
        lda_and_str_rt,
        [
            NewT,
            StrRT(ArgumentRegisterID(0)),
            NewT,
            LdaRT(ArgumentRegisterID(0)),
            Ret
        ],
        |machine: Machine| { assert_eq!(machine.accumulators.t, machine.argument_registers.t[0]) }
    );

    test_instructions_with_locals!(
        lda_and_str_lt,
        [
            NewT,
            StrLT(LocalRegisterID(0)),
            StrRT(ArgumentRegisterID(0)),
            NewT,
            StrRT(ArgumentRegisterID(1)),
            LdaLT(LocalRegisterID(0)),
            Ret
        ],
        reg_count! { T: 1 },
        |machine: Machine| {
            assert_eq!(machine.accumulators.t, machine.argument_registers.t[0]);
            assert_ne!(machine.accumulators.t, machine.argument_registers.t[1]);
        }
    );
}
