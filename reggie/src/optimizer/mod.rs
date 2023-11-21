// This is currently WIP, hence allowing dead code and unused imports.
// TODO: Remove dead code and unused imports.
#![allow(dead_code)]
#![allow(unused_imports)]

use crate::{
    compiler::compile_function,
    ids::{ArgumentRegisterID, GlobalCellID, JmpLabel, LocalRegisterID, SimpleBlockID, StringID},
    keyed_vec::KeyedVec,
    machine::CodeBlock,
    ops::{BranchCondition, Instruction},
    GlobalValues,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockExit {
    End,
    Fallthrough(SimpleBlockID),
    Branch {
        fallthrough: SimpleBlockID,
        to: SimpleBlockID,
        condition: BranchCondition,
    },
}

#[derive(Debug, PartialEq)]
struct SimpleBlock<'a> {
    instrs: &'a [Instruction],
    exit: BlockExit,
}

impl<'a> SimpleBlock<'a> {
    fn unconnected(instrs: &'a [Instruction]) -> Self {
        Self {
            instrs,
            exit: BlockExit::End,
        }
    }
}

fn construct_directed_flow_graph(
    instructions: &[Instruction],
) -> KeyedVec<SimpleBlockID, SimpleBlock<'_>> {
    let (mut blocks, labeled_blocks) = split_simple_blocks(instructions);
    let len: u16 = blocks.len().try_into().unwrap();

    for i in 0..(len - 1) {
        let block = &mut blocks[SimpleBlockID(i)];
        match block.instrs.split_last() {
            Some((Instruction::Jmp(lbl), left)) => {
                block.exit = BlockExit::Fallthrough(labeled_blocks[*lbl]);
                block.instrs = left;
            }
            Some((instr, left)) => match instr.branch_condition() {
                Some((lbl, condition)) => {
                    block.exit = BlockExit::Branch {
                        fallthrough: SimpleBlockID(i + 1),
                        to: labeled_blocks[lbl],
                        condition,
                    };
                    block.instrs = left;
                }
                None => {
                    block.exit = BlockExit::Fallthrough(SimpleBlockID(i + 1));
                }
            },
            _ => {
                block.exit = BlockExit::Fallthrough(SimpleBlockID(i + 1));
            }
        };
    }

    blocks
}

fn split_simple_blocks<'a>(
    instructions: &'a [Instruction],
) -> (
    KeyedVec<SimpleBlockID, SimpleBlock<'a>>,
    KeyedVec<JmpLabel, SimpleBlockID>,
) {
    let mut simple_blocks = KeyedVec::new();
    let mut labeled_blocks = KeyedVec::new();
    let mut labeled = false;
    let mut last_idx = 0;

    for (idx, instr) in instructions.iter().enumerate() {
        if let Instruction::Label = instr {
            let block = &instructions[last_idx..idx];
            let id = simple_blocks.push(SimpleBlock::unconnected(block));
            if labeled {
                labeled_blocks.push(id);
            }
            labeled = true;
            last_idx = idx + 1;
        } else if instr.is_jump() {
            let block = &instructions[last_idx..=idx];
            let id = simple_blocks.push(SimpleBlock::unconnected(block));
            if labeled {
                labeled_blocks.push(id);
            }
            labeled = false;
            last_idx = idx + 1;
        }
    }

    let block = &instructions[last_idx..];
    let id = simple_blocks.push(SimpleBlock::unconnected(block));
    if labeled {
        labeled_blocks.push(id);
    }

    return (simple_blocks, labeled_blocks);
}

pub fn optimize(block: &CodeBlock) -> CodeBlock {
    block.clone()
}

#[test]
fn test_block_split() {
    use Instruction::*;
    let instrs = [
        ConstN,
        NilTest,
        JmpEQ(JmpLabel(0)),
        ConstS(StringID(0)),
        LdaCGl(GlobalCellID(0)),
        Call,
        Label,
        ConstS(StringID(1)),
        LdaCGl(GlobalCellID(0)),
        Call,
        Ret,
    ];
    let graph = construct_directed_flow_graph(&instrs);

    let expected_blocks = [
        SimpleBlock {
            instrs: &[ConstN, NilTest],
            exit: BlockExit::Branch {
                fallthrough: SimpleBlockID(1),
                to: SimpleBlockID(2),
                condition: BranchCondition::EQ,
            },
        },
        SimpleBlock {
            instrs: &[ConstS(StringID(0)), LdaCGl(GlobalCellID(0)), Call],
            exit: BlockExit::Fallthrough(SimpleBlockID(2)),
        },
        SimpleBlock {
            instrs: &[ConstS(StringID(1)), LdaCGl(GlobalCellID(0)), Call, Ret],
            exit: BlockExit::End,
        }
    ];
    let expected_blocks: KeyedVec<SimpleBlockID, _> =
        KeyedVec::from_vec(Vec::from_iter(expected_blocks.into_iter()));

    assert_eq!(expected_blocks, graph);
}

#[test]
fn optimization_result() {
    let decl = luar_syn::lua_parser::function_declaration(
        "
        function foo(a)
            local b = a + 1
            local c = a + 2
            return c
        end
    ",
    )
    .unwrap();
    let input = compile_function(&decl, &mut GlobalValues::default());
    println!("input: {}\n", input);
    println!("output: {}", optimize(&input));
}
