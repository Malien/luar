use crate::{ids::BlockID, lua_format, trace_execution, ArithmeticOperator};

use super::{
    ids::{ArgumentRegisterID, LocalRegisterID},
    machine::{Machine, ProgramCounter, TestFlag},
    ops::Instruction,
    ArithmeticError, EvalError, InvalidLuaKey, LuaKey, LuaValue, NativeFunction,
    NativeFunctionKind, TableRef, TableValue, TypeError,
};
use std::{borrow::Borrow, cmp::Ordering};

macro_rules! register_of {
    ($machine:expr, AD) => {
        $machine.accumulators.d
    };
    ($machine:expr, AI) => {
        $machine.accumulators.i
    };
    ($machine:expr, AF) => {
        $machine.accumulators.f
    };
    ($machine:expr, AS) => {
        $machine.accumulators.s
    };
    ($machine:expr, AC) => {
        $machine.accumulators.c
    };
    ($machine:expr, AT) => {
        $machine.accumulators.t
    };

    ($machine:expr, RD, $reg:ident) => {
        $machine.argument_registers.d[($reg as ArgumentRegisterID).0 as usize]
    };
    ($machine:expr, RI, $reg:ident) => {
        $machine.argument_registers.i[($reg as ArgumentRegisterID).0 as usize]
    };
    ($machine:expr, RT, $reg:ident) => {
        $machine.argument_registers.t[($reg as ArgumentRegisterID).0 as usize]
    };

    ($machine:expr, RD0) => {
        $machine.argument_registers.d[0]
    };
    ($machine:expr, RD1) => {
        $machine.argument_registers.d[1]
    };
    ($machine:expr, RD2) => {
        $machine.argument_registers.d[2]
    };
    ($machine:expr, RD3) => {
        $machine.argument_registers.d[3]
    };
    ($machine:expr, RD4) => {
        $machine.argument_registers.d[4]
    };

    ($machine:expr, RT0) => {
        $machine.argument_registers.t[0]
    };
    ($machine:expr, RT1) => {
        $machine.argument_registers.t[1]
    };
}

pub(crate) fn execute(machine: &mut Machine, block_id: BlockID) -> Result<(), EvalError> {
    machine.program_counter = ProgramCounter {
        block: block_id,
        position: 0,
    };

    let mut block = &machine.code_blocks[machine.program_counter.block];
    let mut position = &mut machine.program_counter.position;
    let mut frame = machine.stack.push(
        &block.meta,
        ProgramCounter {
            block: block_id,
            position: 0,
        },
    );

    macro_rules! register {
        (LD, $reg:ident) => {
            frame.get_dyn($reg as LocalRegisterID)
        };
        (LI, $reg:ident) => {
            frame.get_int($reg as LocalRegisterID)
        };
        (LT, $reg:ident) => {
            frame.get_table($reg as LocalRegisterID)
        };

        ($rest:tt) => {
            register_of!(machine, $rest)
        };
        ($first:tt, $rest:tt) => {
            register_of!(machine, $first, $rest)
        };
    }

    loop {
        let instr = block.instructions[*position as usize];
        match instr {
            Instruction::Ret => {
                machine.program_counter = frame.return_addr();
                position = &mut machine.program_counter.position;

                let release_handle = frame.release();
                unsafe { machine.stack.pop(release_handle) };
                if machine.stack.is_empty() {
                    return Ok(());
                }
                block = &machine.code_blocks[machine.program_counter.block];
                // SAFETY: We keep track of the stack frames, and guarantee
                //         first-come first-serve ordering of stack frames.
                frame = unsafe { machine.stack.restore(&block.meta) };

                trace_execution!(
                    "ret back to {:?} {}",
                    frame.return_addr.block,
                    block
                        .meta
                        .debug_name
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or_default()
                );
            }
            Instruction::ConstI(value) => {
                register!(AI) = value;
                *position += 1;
            }
            Instruction::WrapI => {
                register!(AD) = LuaValue::Int(register!(AI));
                *position += 1;
            }
            Instruction::StrRD(reg) => {
                register!(RD, reg) = register!(AD).clone();
                *position += 1;
            }
            Instruction::LdaRD(reg) => {
                register!(AD) = register!(RD, reg).clone();
                *position += 1;
            }
            Instruction::StrLD(reg) => {
                *register!(LD, reg) = register!(AD).clone();
                *position += 1;
            }
            Instruction::LdaLD(reg) => {
                register!(AD) = register!(LD, reg).clone();
                *position += 1;
            }
            Instruction::DAddR(reg) => {
                register!(AD) = add_dyn(&register!(AD), &register!(RD, reg))?;
                *position += 1;
            }
            Instruction::DAddL(reg) => {
                register!(AD) = add_dyn(&register!(AD), &register!(LD, reg))?;
                *position += 1;
            }
            Instruction::ConstN => {
                register!(AD) = LuaValue::Nil;
                *position += 1;
            }
            Instruction::ConstF(value) => {
                register!(AF) = value;
                *position += 1;
            }
            Instruction::WrapF => {
                register!(AD) = LuaValue::Float(register!(AF));
                *position += 1;
            }
            Instruction::ConstS(string_id) => {
                register!(AS) = block.meta.const_strings[string_id].clone();
                *position += 1;
            }
            Instruction::WrapS => {
                register!(AD) = LuaValue::String(register!(AS).clone());
                *position += 1;
            }
            Instruction::ConstC(local_block_id) => {
                register!(AC) = machine.code_blocks.blocks_of_module(block.module)[local_block_id];
                *position += 1;
            }
            Instruction::WrapC => {
                register!(AD) = LuaValue::Function(register!(AC));
                *position += 1;
            }
            Instruction::LdaDGl(cell_id) => {
                register!(AD) = machine.global_values.value_of_cell(cell_id).clone();
                *position += 1;
            }
            Instruction::EqTestRD(reg) => {
                machine.test_flag = TestFlag::from_bool(register!(AD) == register!(RD, reg));
                *position += 1;
            }
            Instruction::EqTestLD(reg) => {
                machine.test_flag = TestFlag::from_bool(&mut register!(AD) == register!(LD, reg));
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
                if let TestFlag::EQ = machine.test_flag {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpNE(jmp_label) => {
                if let TestFlag::NE = machine.test_flag {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpLT(jmp_label) => {
                if let TestFlag::LT = machine.test_flag {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpGT(jmp_label) => {
                if let TestFlag::GT = machine.test_flag {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpLE(jmp_label) => {
                if machine.test_flag == TestFlag::LT || machine.test_flag == TestFlag::EQ {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::JmpGE(jmp_label) => {
                if machine.test_flag == TestFlag::GT || machine.test_flag == TestFlag::EQ {
                    *position = block.meta.label_mappings[jmp_label];
                } else {
                    *position += 1;
                }
            }
            Instruction::StrDGl(cell) => {
                machine.global_values.set_cell(cell, register!(AD).clone());
                *position += 1;
            }
            Instruction::StrVC => {
                machine.value_count = register!(AI).try_into().unwrap();
                *position += 1;
            }
            Instruction::DCall => match register!(AD).clone() {
                LuaValue::Function(block_id) => {
                    let new_block = &machine.code_blocks[block_id];
                    trace_execution!(
                        "d_call into {block_id:?} {}",
                        new_block
                            .meta
                            .debug_name
                            .as_ref()
                            .map(String::as_str)
                            .unwrap_or_default()
                    );
                    frame = machine.stack.push(
                        &new_block.meta,
                        ProgramCounter {
                            position: *position + 1,
                            block: machine.program_counter.block,
                        },
                    );
                    block = new_block;
                    *position = 0;
                    machine.program_counter.block = block_id;
                }
                LuaValue::NativeFunction(NativeFunction(native_fn_kind)) => {
                    match native_fn_kind.borrow() {
                        NativeFunctionKind::Dyn(dyn_fn) => {
                            trace_execution!(
                                "d_call into native function {:p}",
                                dyn_fn as *const _
                            );
                            dyn_fn.call(&mut machine.argument_registers, machine.value_count)?;
                            machine.value_count = dyn_fn.return_count();
                        } // NativeFunctionKind::OverloadSet(_) => {
                          //     todo!("Cannot call native functions defined with overload sets yet")
                          // }
                    };
                    *position += 1;
                }
                _val => {
                    trace_execution!("d_call {_val}");
                    return Err(EvalError::from(TypeError::IsNotCallable(
                        register!(AD).clone(),
                    )));
                }
            },
            Instruction::LdaProt(reg) => {
                register!(AD) = if machine.value_count > reg.0 {
                    register!(RD, reg).clone()
                } else {
                    LuaValue::Nil
                };
                *position += 1;
            }
            Instruction::TypedCall => {
                let new_block = &machine.code_blocks[register!(AC)];
                trace_execution!(
                    "typed_call into {:?} {}",
                    register!(AC),
                    new_block
                        .meta
                        .debug_name
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or_default()
                );
                frame = machine.stack.push(
                    &new_block.meta,
                    ProgramCounter {
                        position: *position + 1,
                        block: machine.program_counter.block,
                    },
                );
                block = new_block;
                *position = 0;
                machine.program_counter.block = register!(AC);
            }
            Instruction::RDShiftRight => {
                machine
                    .argument_registers
                    .d
                    .rotate_right((register!(AI) as u16) as usize);
                *position += 1;
            }
            Instruction::LdaVC => {
                register!(AI) = machine.value_count as i32;
                *position += 1;
            }
            Instruction::IAddR(reg) => {
                register!(AI) += register!(RI, reg);
                *position += 1;
            }
            Instruction::IAddL(reg) => {
                register!(AI) += *register!(LI, reg);
                *position += 1;
            }
            Instruction::StrLI(reg) => {
                *register!(LI, reg) = register!(AI);
                *position += 1;
            }
            Instruction::LdaLI(reg) => {
                register!(AI) = *register!(LI, reg);
                *position += 1;
            }
            Instruction::StrRI(reg) => {
                register!(RI, reg) = register!(AI);
                *position += 1;
            }
            Instruction::LdaRI(reg) => {
                register!(AI) = register!(RI, reg);
                *position += 1;
            }
            Instruction::NilTest => {
                machine.test_flag = TestFlag::from_bool(register!(AD) == LuaValue::Nil);
                *position += 1;
            }
            Instruction::DSubR(reg) => {
                register!(AD) = sub_dyn(&register!(AD), &register!(RD, reg))?;
                *position += 1;
            }
            Instruction::DSubL(reg) => {
                register!(AD) = sub_dyn(&register!(AD), &register!(LD, reg))?;
                *position += 1;
            }
            Instruction::NewT => {
                register!(AT) = Some(TableRef::from(TableValue::new()));
                *position += 1;
            }
            Instruction::StrRT(reg) => {
                register!(RT, reg) = register!(AT).clone();
                *position += 1;
            }
            Instruction::LdaRT(reg) => {
                register!(AT) = register!(RT, reg).clone();
                *position += 1;
            }
            Instruction::LdaLT(reg) => {
                register!(AT) = register!(LT, reg).clone();
                *position += 1;
            }
            Instruction::StrLT(reg) => {
                *register!(LT, reg) = register!(AT).clone();
                *position += 1;
            }
            Instruction::PushD => {
                let table = register!(AT).as_mut().unwrap();
                table.push(register!(AD).clone());
                *position += 1;
            }
            Instruction::AssocASD => {
                let table = register!(AT).as_mut().unwrap();
                table.assoc_str(register!(AS).clone(), register!(AD).clone());
                *position += 1;
            }
            Instruction::CastT => {
                machine.test_flag = if let LuaValue::Table(ref table) = register!(AD) {
                    register!(AT) = Some(table.clone());
                    TestFlag::EQ
                } else {
                    TestFlag::NE
                };
                *position += 1;
            }
            Instruction::TablePropertyLookupError => {
                return Err(EvalError::from(TypeError::CannotAccessProperty {
                    property: register!(AS).clone(),
                    of: std::mem::replace(&mut register!(AD), LuaValue::Nil),
                }))
            }
            Instruction::TableMemberLookupErrorR(reg) => {
                return Err(EvalError::from(TypeError::CannotAccessMember {
                    member: std::mem::replace(&mut register!(RD, reg), LuaValue::Nil),
                    of: std::mem::replace(&mut register!(AD), LuaValue::Nil),
                }))
            }
            Instruction::TableMemberLookupErrorL(reg) => {
                return Err(EvalError::from(TypeError::CannotAccessMember {
                    member: std::mem::replace(&mut register!(LD, reg), LuaValue::Nil),
                    of: std::mem::replace(&mut register!(AD), LuaValue::Nil),
                }))
            }
            Instruction::WrapT => {
                register!(AD) = LuaValue::Table(register!(AT).as_ref().unwrap().clone());
                *position += 1;
            }
            Instruction::LdaAssocAS => {
                register!(AD) = machine
                    .accumulators
                    .t
                    .as_mut()
                    .unwrap()
                    .get_str_assoc(register!(AS).clone());
                *position += 1;
            }
            Instruction::LdaAssocAD => {
                match LuaKey::try_from(register!(AD).clone()) {
                    Ok(key) => {
                        register!(AD) = register!(AT).as_mut().unwrap().get(&key);
                    }
                    Err(_) => {
                        register!(AD) = LuaValue::Nil;
                    }
                }
                *position += 1;
            }
            Instruction::DMulR(reg) => {
                register!(AD) = mul_dyn(&register!(AD), &register!(RD, reg))?;
                *position += 1;
            }
            Instruction::DMulL(reg) => {
                register!(AD) = mul_dyn(&register!(AD), &register!(LD, reg))?;
                *position += 1;
            }
            Instruction::DDivR(reg) => {
                register!(AD) = div_dyn(&register!(AD), &register!(RD, reg))?;
                *position += 1;
            }
            Instruction::DDivL(reg) => {
                register!(AD) = div_dyn(&register!(AD), &register!(LD, reg))?;
                *position += 1;
            }
            Instruction::AssocRD(reg) => {
                let value = register!(RD, reg).clone();
                let key = match LuaKey::try_from(register!(AD).clone()) {
                    Ok(key) => key,
                    Err(InvalidLuaKey::Nil) => {
                        return Err(EvalError::from(TypeError::NilAssign(value)))
                    }
                    Err(InvalidLuaKey::NaN) => {
                        return Err(EvalError::from(TypeError::NaNAssign(value)))
                    }
                };
                register!(AT).as_mut().unwrap().set(key, value);
                *position += 1;
            }
            Instruction::AssocLD(reg) => {
                let value = register!(LD, reg).clone();
                let key = match LuaKey::try_from(register!(AD).clone()) {
                    Ok(key) => key,
                    Err(InvalidLuaKey::Nil) => {
                        return Err(EvalError::from(TypeError::NilAssign(value)))
                    }
                    Err(InvalidLuaKey::NaN) => {
                        return Err(EvalError::from(TypeError::NaNAssign(value)))
                    }
                };
                register!(AT).as_mut().unwrap().set(key, value);
                *position += 1;
            }
            Instruction::TablePropertyAssignError => {
                return Err(EvalError::from(TypeError::CannotAssignProperty {
                    property: register!(AS).clone(),
                    of: std::mem::replace(&mut register!(AD), LuaValue::Nil),
                }))
            }
            Instruction::TableMemberAssignErrorR(reg) => {
                return Err(EvalError::from(TypeError::CannotAssignMember {
                    member: std::mem::replace(&mut register!(RD, reg), LuaValue::Nil),
                    of: std::mem::replace(&mut register!(AD), LuaValue::Nil),
                }))
            }
            Instruction::TableMemberAssignErrorL(reg) => {
                return Err(EvalError::from(TypeError::CannotAssignMember {
                    member: std::mem::replace(&mut register!(LD, reg), LuaValue::Nil),
                    of: std::mem::replace(&mut register!(AD), LuaValue::Nil),
                }))
            }
            Instruction::NegD => {
                neg_dyn_accumulator(&mut register!(AD))?;
                *position += 1;
            }
            Instruction::TestRD(reg) => {
                let ordering = LuaValue::partial_cmp(&register!(AD), &register!(RD, reg));
                machine.test_flag = cmp_test_flags(ordering);
                *position += 1;
            }
            Instruction::TestLD(reg) => {
                let lhs = &mut register!(AD);
                let rhs = &mut register!(LD, reg);
                if !lhs.is_comparable() || !rhs.is_comparable() {
                    return Err(EvalError::from(TypeError::Ordering {
                        lhs: std::mem::replace(lhs, LuaValue::Nil),
                        rhs: std::mem::replace(rhs, LuaValue::Nil),
                        op: None,
                    }));
                }
                let ordering = LuaValue::partial_cmp(lhs, rhs);
                machine.test_flag = cmp_test_flags(ordering);
                *position += 1;
            }
            Instruction::DConcatR(reg) => {
                register!(AD) = dyn_concat(&register!(AD), &register!(RD, reg))?;
                *position += 1;
            }
            Instruction::DConcatL(reg) => {
                register!(AD) = dyn_concat(&register!(AD), &register!(LD, reg))?;
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
            Instruction::TestLF(_) => todo!(),
            Instruction::TestLS(_) => todo!(),
            Instruction::TestLI(_) => todo!(),
            Instruction::TestLT(_) => todo!(),
            Instruction::TestLC(_) => todo!(),
            Instruction::TestLU(_) => todo!(),
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
            Instruction::NegF => todo!(),
            Instruction::NegI => todo!(),
        }
    }
}

fn cmp_test_flags(ordering: Option<Ordering>) -> TestFlag {
    match ordering {
        Some(Ordering::Equal) => TestFlag::EQ,
        Some(Ordering::Less) => TestFlag::LT,
        Some(Ordering::Greater) => TestFlag::GT,
        None => TestFlag::NE,
    }
}

fn neg_dyn_accumulator(accumulator: &mut LuaValue) -> Result<(), EvalError> {
    match accumulator {
        LuaValue::Int(ref mut int) => {
            *int = -*int;
            Ok(())
        }
        LuaValue::Float(ref mut float) => {
            *float = -*float;
            Ok(())
        }
        LuaValue::String(ref str) => match str.parse::<f64>() {
            Ok(value) => {
                *accumulator = LuaValue::Float(-value);
                Ok(())
            }
            Err(_) => Err(EvalError::from(TypeError::Arithmetic(
                ArithmeticError::UnaryMinus(std::mem::replace(accumulator, LuaValue::Nil)),
            ))),
        },
        accumulator => Err(EvalError::from(TypeError::Arithmetic(
            ArithmeticError::UnaryMinus(std::mem::replace(accumulator, LuaValue::Nil)),
        ))),
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

macro_rules! dyn_concat_of {
    ($lhs:expr, $rhs:expr; $(($left_pat:ident, $right_pat:ident),)*) => {
        match ($lhs, $rhs) {
            $((LuaValue::$left_pat(lhs), LuaValue::$right_pat(rhs)) =>
              Some(lua_format!("{lhs}{rhs}"))
            ,)*
            _ => None,
        }
    };
}

fn dyn_concat(lhs: &LuaValue, rhs: &LuaValue) -> Result<LuaValue, TypeError> {
    dyn_concat_of! {
        lhs, rhs;

        (Int, Int),
        (Int, Float),
        (Int, String),
        (Float, Int),
        (Float, Float),
        (Float, String),
        (String, Int),
        (String, Float),
        (String, String),
    }
    .map(LuaValue::String)
    .ok_or_else(|| TypeError::StringConcat {
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    })
}

#[cfg(test)]
mod test {
    use crate::{
        call_block,
        compiler::CompiledModule,
        ids::{ArgumentRegisterID, JmpLabel, LocalBlockID, LocalRegisterID, StringID},
        keyed_vec::keyed_vec,
        machine::{
            CodeBlock, Machine,
            TestFlag::{self, *},
        },
        meta::{reg_count, CodeMeta, LocalRegCount},
        ops::Instruction::{self, *},
        EvalError, LuaValue, NativeFunction, Strict, TypeError,
    };
    use ntest::timeout;

    macro_rules! test_instructions_with_meta {
        (
            name: $name:ident,
            code: [$($instr: expr),*$(,)?],
            meta: $meta:expr,
            post_condition: $post_condition: expr
        ) => {
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
        (
            name: $name: ident,
            code: [$($instr: expr),*$(,)?],
            locals: $locals: expr,
            post_condition: $post_condition: expr
        ) => {
            test_instructions_with_meta! {
                name: $name,
                code: [$($instr,)*],
                meta: CodeMeta {
                    arg_count: 0.into(),
                    local_count: $locals,
                    return_count: 0.into(),
                    ..Default::default()
                },
                post_condition: $post_condition
            }
        };
    }

    macro_rules! test_instructions {
        ($name: ident, [$($instr: expr),*$(,)?], $post_condition: expr) => {
            test_instructions_with_locals! {
                name: $name,
                code: [$($instr,)*],
                locals: LocalRegCount::default(),
                post_condition: $post_condition
            }
        };
        (name: $name:ident, code: $code:tt, post_condition: $post_condition: expr) => {
            test_instructions!($name, $code, $post_condition);
        };
    }

    macro_rules! test_instructions_with_strings {
        (
            name: $name: ident,
            code: [$($instr: expr),*],
            strings: [$($strings: expr),*],
            post_condition: $post_condition:expr
        ) => {
            test_instructions_with_meta! {
                name: $name,
                code: [$($instr,)*],
                meta: CodeMeta {
                    arg_count: 0.into(),
                    return_count: 0.into(),
                    const_strings: $crate::keyed_vec::keyed_vec![
                        $($strings.into(),)*
                    ],
                    ..Default::default()
                },
                post_condition: $post_condition
            }
        };
    }

    test_instructions!(ret_fn_call, [Ret], |_| {});

    test_instructions!(const_i, [ConstI(42), Ret], |machine: Machine| {
        assert_eq!(register_of!(machine, AI), 42);
    });

    test_instructions!(wrap_i, [ConstI(42), WrapI, Ret], |machine: Machine| {
        assert_eq!(register_of!(machine, AD), LuaValue::Int(42));
    });

    test_instructions!(
        str_rd,
        [ConstI(42), WrapI, StrRD(ArgumentRegisterID(0)), Ret],
        |machine: Machine| { assert_eq!(register_of!(machine, RD0), LuaValue::Int(42)) }
    );

    test_instructions_with_locals! {
        name: str_and_lda_ld,
        code: [
            ConstI(42),
            WrapI,
            StrLD(LocalRegisterID(0)),
            ConstI(69),
            WrapI,
            LdaLD(LocalRegisterID(0)),
            Ret
        ],
        locals: reg_count! { D: 1 },
        post_condition: |machine: Machine| { assert_eq!(register_of!(machine, AD), LuaValue::Int(42)) }
    }

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
        |machine: Machine| { assert_eq!(register_of!(machine, AD), LuaValue::Int(42)) }
    );

    test_instructions_with_locals! {
        name: plus_1_and_2_local_regs,
        code: [
            ConstI(1),
            WrapI,
            StrLD(LocalRegisterID(0)),
            ConstI(2),
            WrapI,
            DAddL(LocalRegisterID(0)),
            StrRD(ArgumentRegisterID(0)),
            Ret,
        ],
        locals: reg_count! { D: 1 },
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, RD0).coerce_to_f64(), Some(3.0));
        }
    }

    test_instructions_with_locals! {
        name: plus_1_and_2_arg_regs,
        code: [
            ConstI(1),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstI(2),
            WrapI,
            DAddR(ArgumentRegisterID(0)),
            StrRD(ArgumentRegisterID(0)),
            Ret,
        ],
        locals: reg_count! { D: 1 },
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, RD0).coerce_to_f64(), Some(3.0));
        }
    }

    test_instructions!(
        const_n,
        [ConstI(42), WrapI, ConstN, Ret],
        |machine: Machine| { assert_eq!(register_of!(machine, RD0), LuaValue::Nil) }
    );

    test_instructions!(const_f, [ConstF(42.4), Ret], |machine: Machine| {
        assert_eq!(register_of!(machine, AF), 42.4)
    });

    test_instructions!(wrap_f, [ConstF(42.4), WrapF, Ret], |machine: Machine| {
        assert_eq!(register_of!(machine, AD), LuaValue::Float(42.4))
    });

    test_instructions_with_strings! {
        name: const_s,
        code: [ConstS(StringID(0)), Ret],
        strings: ["hello"],
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AS), "hello")
        }
    }

    test_instructions_with_strings! {
        name: wrap_s,
        code: [ConstS(StringID(0)), WrapS, Ret],
        strings: ["hello"],
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AD), LuaValue::string("hello"))
        }
    }

    #[test]
    fn after_execution_error_machine_stack_is_cleared() {
        let mut machine = Machine::new();

        let block_id = machine.code_blocks.add_module(CompiledModule {
            blocks: keyed_vec![
                CodeBlock {
                    meta: CodeMeta {
                        arg_count: 0.into(),
                        return_count: 0.into(),
                        local_count: reg_count! { D: 1 },
                        ..Default::default()
                    },
                    instructions: vec![ConstN, DCall, Ret]
                },
                CodeBlock {
                    meta: CodeMeta {
                        arg_count: 0.into(),
                        return_count: 0.into(),
                        ..Default::default()
                    },
                    instructions: vec![ConstC(LocalBlockID(0)), TypedCall, Ret]
                }
            ],
            top_level: CodeBlock {
                meta: CodeMeta {
                    arg_count: 0.into(),
                    return_count: 0.into(),
                    ..Default::default()
                },
                instructions: vec![ConstC(LocalBlockID(1)), TypedCall, Ret],
            },
        });

        let result = call_block::<()>(block_id, &mut machine);
        match result {
            Err(EvalError::TypeError(err)) => {
                assert_eq!(*err, TypeError::IsNotCallable(LuaValue::Nil))
            }
            _ => panic!("Expected ExecutionError, got {:?}", result),
        }
        assert!(machine.stack.is_empty(), "Stack is not empty");
    }

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
        assert!(register_of!(machine, AD).total_eq(&value));
    }

    #[cfg(feature = "quickcheck")]
    #[quickcheck]
    // #[timeout(5000)]
    fn eq_test_d(lhs: LuaValue, rhs: LuaValue) {
        let mut machine = Machine::new();
        let expected = TestFlag::from_bool(lhs == rhs);
        register_of!(machine, RD0) = lhs;
        register_of!(machine, RD1) = rhs;
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
        assert_eq!(machine.test_flag, expected);
    }

    test_instructions_with_meta! {
        name: jmp,
        code: [ConstI(1), Jmp(JmpLabel(0)), ConstI(2), Label, Ret],
        meta: CodeMeta {
            arg_count: 0.into(),
            label_mappings: keyed_vec![3],
            return_count: 0.into(),
            ..Default::default()
        },
        post_condition: |machine: Machine| { assert_eq!(register_of!(machine, AI), 1) }
    }

    static CONDITIONAL_JMP_BEHAVIOR: [(fn(JmpLabel) -> Instruction, &[TestFlag]); 6] = [
        (JmpEQ, &[EQ]),
        (JmpNE, &[NE]),
        (JmpLT, &[LT]),
        (JmpGT, &[GT]),
        (JmpLE, &[EQ, LT]),
        (JmpGE, &[EQ, GT]),
    ];

    static FLAGS_PERMUTATION: [TestFlag; 4] = [EQ, NE, LT, GT];

    #[test]
    fn conditional_jumps() {
        let mut machine = Machine::new();
        for (jmp_instr, triggered_flags) in CONDITIONAL_JMP_BEHAVIOR {
            for flag in FLAGS_PERMUTATION {
                let block_id = machine.code_blocks.add_top_level_block(CodeBlock {
                    meta: CodeMeta {
                        arg_count: 0.into(),
                        return_count: 0.into(),
                        label_mappings: keyed_vec![3],
                        ..Default::default()
                    },
                    instructions: vec![ConstI(1), jmp_instr(JmpLabel(0)), ConstI(2), Label, Ret],
                });
                machine.test_flag = flag;
                call_block::<Strict<()>>(block_id, &mut machine).unwrap();
                let expected_value = if triggered_flags.contains(&flag) {
                    1
                } else {
                    2
                };
                assert_eq!(
                    register_of!(machine, AI),
                    expected_value,
                    "While executing {} with triggered flags: {:?} and actual flag {:?}",
                    jmp_instr(JmpLabel(0)),
                    triggered_flags,
                    flag
                );
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
        use crate::assert_type_error;

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

        assert_type_error!(TypeError::IsNotCallable(_), res);
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
            NativeFunction::new(|| Result::<(), EvalError>::Err(EvalError::AssertionError(None)));
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
        assert!(matches!(res, Err(EvalError::AssertionError(None))));
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

        let block1_id = register_of!(machine, AD).unwrap_lua_function();
        let block2_id = register_of!(machine, AC);

        assert_eq!(
            machine.code_blocks[block1_id].instructions,
            block1.instructions
        );
        assert_eq!(
            machine.code_blocks[block2_id].instructions,
            block2.instructions
        );
    }

    test_instructions! {
        name: lda_prot,
        code: [
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
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, RD0), LuaValue::Int(42));
            assert_eq!(register_of!(machine, RD1), LuaValue::Nil);
        }
    }

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

    test_instructions! {
        name: rd_shift_right,
        code: [
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
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, RD2), LuaValue::Int(1));
            assert_eq!(register_of!(machine, RD3), LuaValue::Int(2));
            assert_eq!(register_of!(machine, RD4), LuaValue::Int(3));
        }
    }

    test_instructions! {
        name: lda_vc, code: [ConstI(69), StrVC, ConstI(42), LdaVC, Ret],
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AI), 69);
        }
    }

    test_instructions_with_locals! {
        name: str_li_and_lda_li,
        code: [
            ConstI(69),
            StrLI(LocalRegisterID(0)),
            ConstI(42),
            LdaLI(LocalRegisterID(0)),
            Ret
        ],
        locals: reg_count! { I: 1 },
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AI), 69);
        }
    }

    test_instructions! {
        name: str_ri_and_lda_ri,
        code: [
            ConstI(69),
            StrRI(ArgumentRegisterID(0)),
            ConstI(42),
            LdaRI(ArgumentRegisterID(0)),
            Ret
        ],
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AI), 69);
        }
    }

    test_instructions_with_locals! {
        name: i_add,
        code: [
            ConstI(68000),
            StrLI(LocalRegisterID(0)),
            ConstI(420),
            StrRI(ArgumentRegisterID(0)),
            ConstI(1000),
            IAddL(LocalRegisterID(0)),
            IAddR(ArgumentRegisterID(0)),
            Ret
        ],
        locals: reg_count! { I: 1 },
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AI), 69420);
        }
    }

    test_instructions! {
        name: nil_test_nil,
        code: [
            ConstI(1),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstI(2),
            EqTestRD(ArgumentRegisterID(0)),
            ConstN,
            NilTest,
            Ret
        ],
        post_condition: |machine: Machine| {
            assert!(machine.test_flag.test_succeeded());
        }
    }

    test_instructions! {
        name: nil_test_non_nill,
        code: [
            ConstI(1),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstI(1),
            EqTestRD(ArgumentRegisterID(0)),
            NilTest,
            Ret
        ],
        post_condition: |machine: Machine| {
            assert!(machine.test_flag.test_failed());
        }
    }

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
    //     reg_count! { D: 1, },
    //     |machine: Machine| {
    //         assert_eq!(register_of!(machine, AD), LuaValue::Int(69));
    //     }
    // );

    test_instructions! {
        name: new_and_str_rt,
        code: [NewT, StrRT(ArgumentRegisterID(0)), NewT, Ret],
        post_condition: |machine: Machine| {
            assert!(register_of!(machine, AT).is_some());
            assert!(register_of!(machine, RT0).is_some());
            assert_ne!(register_of!(machine, RT0), register_of!(machine, AT));
        }
    }

    test_instructions! {
        name: lda_and_str_rt,
        code: [
            NewT,
            StrRT(ArgumentRegisterID(0)),
            NewT,
            LdaRT(ArgumentRegisterID(0)),
            Ret
        ],
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AT), register_of!(machine, RT0))
        }
    }

    test_instructions_with_locals! {
        name: lda_and_str_lt,
        code: [
            NewT,
            StrLT(LocalRegisterID(0)),
            StrRT(ArgumentRegisterID(0)),
            NewT,
            StrRT(ArgumentRegisterID(1)),
            LdaLT(LocalRegisterID(0)),
            Ret
        ],
        locals: reg_count! { T: 1 },
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AT), register_of!(machine, RT0));
            assert_ne!(register_of!(machine, AT), register_of!(machine, RT1));
        }
    }

    test_instructions_with_strings! {
        name: d_concat_r_strings,
        code: [
            ConstS(StringID(1)),
            WrapS,
            StrRD(ArgumentRegisterID(0)),
            ConstS(StringID(0)),
            WrapS,
            DConcatR(ArgumentRegisterID(0)),
            Ret
        ],
        strings: ["hello", "world"],
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AD), LuaValue::string("helloworld"))
        }
    }

    test_instructions! {
        name: d_concat_r_numbers,
        code: [
            ConstI(42),
            WrapI,
            StrRD(ArgumentRegisterID(0)),
            ConstF(69.288),
            WrapF,
            DConcatR(ArgumentRegisterID(0)),
            Ret
        ],
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AD), LuaValue::string("69.28842"))
        }
    }

    test_instructions_with_meta! {
        name: d_concat_l_strings,
        code: [
            ConstS(StringID(1)),
            WrapS,
            StrLD(LocalRegisterID(0)),
            ConstS(StringID(0)),
            WrapS,
            DConcatL(LocalRegisterID(0)),
            Ret
        ],
        meta: CodeMeta {
            local_count: reg_count! { D: 1 },
            const_strings: keyed_vec!["hello".into(), "world".into()],
            ..Default::default()
        },
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AD), LuaValue::string("helloworld"))
        }
    }

    test_instructions_with_locals! {
        name: d_concat_l_numbers,
        code: [
            ConstI(42),
            WrapI,
            StrLD(LocalRegisterID(0)),
            ConstF(69.288),
            WrapF,
            DConcatL(LocalRegisterID(0)),
            Ret
        ],
        locals: reg_count! { D: 1 },
        post_condition: |machine: Machine| {
            assert_eq!(register_of!(machine, AD), LuaValue::string("69.28842"))
        }
    }
}
