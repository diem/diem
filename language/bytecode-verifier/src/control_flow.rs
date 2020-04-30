// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifies control flow. The following properties are
//! ensured:
//! - All forward jumps do not enter into the middle of a loop
//! - All "breaks" (forward, loop-exiting jumps) go to the "end" of the loop
//! - All "continues" (back jumps in a loop) are only to the current loop
use libra_types::vm_error::{StatusCode, VMStatus};
use std::{collections::HashSet, convert::TryInto};
use vm::{
    errors::{err_at_offset, VMResult},
    file_format::{Bytecode, CodeOffset, CompiledModule, FunctionDefinition},
};

pub fn verify(_module: &CompiledModule, function_definition: &FunctionDefinition) -> VMResult<()> {
    let code = match &function_definition.code {
        Some(code) => &code.code,
        None => return Ok(()),
    };

    // check fall trhough
    // Check to make sure that the bytecode vector ends with a branching instruction.
    match code.last() {
        None => return Err(VMStatus::new(StatusCode::EMPTY_CODE_UNIT)),
        Some(last) if !last.is_unconditional_branch() => {
            return Err(err_at_offset(
                StatusCode::INVALID_FALL_THROUGH,
                code.len() - 1,
            ))
        }
        Some(_) => (),
    }

    // check jumps
    let context = &ControlFlowVerifier { code };
    let labels = instruction_labels(context);
    check_jumps(context, labels)
}

#[derive(Clone, Copy)]
enum Label {
    Loop { last_continue: u16 },
    Code,
}

struct ControlFlowVerifier<'a> {
    code: &'a Vec<Bytecode>,
}

impl<'a> ControlFlowVerifier<'a> {
    fn code(&self) -> impl Iterator<Item = (CodeOffset, &'a Bytecode)> {
        self.code
            .iter()
            .enumerate()
            .map(|(idx, instr)| (idx.try_into().unwrap(), instr))
    }

    fn labeled_code<'b: 'a>(
        &self,
        labels: &'b [Label],
    ) -> impl Iterator<Item = (CodeOffset, &'a Bytecode, &'b Label)> {
        self.code()
            .zip(labels)
            .map(|((i, instr), lbl)| (i, instr, lbl))
    }
}

fn instruction_labels(context: &ControlFlowVerifier) -> Vec<Label> {
    let mut labels: Vec<Label> = (0..context.code.len()).map(|_| Label::Code).collect();
    let mut loop_continue = |loop_idx: CodeOffset, last_continue: CodeOffset| {
        labels[loop_idx as usize] = Label::Loop { last_continue }
    };
    for (i, instr) in context.code() {
        match instr {
            // Back jump/"continue"
            Bytecode::Branch(prev) | Bytecode::BrTrue(prev) | Bytecode::BrFalse(prev)
                if *prev <= i =>
            {
                loop_continue(*prev, i)
            }
            _ => (),
        }
    }
    labels
}

// Ensures the invariant:
//   - All forward jumps do not enter into the middle of a loop
//   - All "breaks" go to the "end" of the loop
//   - All back jumps are only to the current loop
fn check_jumps(context: &ControlFlowVerifier, labels: Vec<Label>) -> VMResult<()> {
    // All back jumps are only to the current loop
    check_continues(context, &labels)?;
    // All "breaks" go to the "end" of the loop
    check_breaks(context, &labels)?;
    // All forward jumps do not enter into the middle of a loop
    check_no_loop_splits(context, &labels)
}

fn check_code<F: FnMut(&Vec<(CodeOffset, CodeOffset)>, CodeOffset, &Bytecode) -> VMResult<()>>(
    context: &ControlFlowVerifier,
    labels: &[Label],
    mut check: F,
) -> VMResult<()> {
    let mut loop_stack: Vec<(CodeOffset, CodeOffset)> = vec![];
    for (i, instr, label) in context.labeled_code(labels) {
        // Add loop to stack
        if let Label::Loop { last_continue } = label {
            loop_stack.push((i, *last_continue));
        }

        check(&loop_stack, i, instr)?;

        // Pop if last continue
        match instr {
            // Back jump/"continue"
            Bytecode::Branch(j) | Bytecode::BrTrue(j) | Bytecode::BrFalse(j) if *j <= i => {
                let (_cur_loop, last_continue) = loop_stack.last().unwrap();
                if i == *last_continue {
                    loop_stack.pop();
                }
            }
            _ => (),
        }
    }
    Ok(())
}

// All back jumps are only to the current loop
fn check_continues(context: &ControlFlowVerifier, labels: &[Label]) -> VMResult<()> {
    check_code(context, labels, |loop_stack, i, instr| {
        match instr {
            // Back jump/"continue"
            Bytecode::Branch(j) | Bytecode::BrTrue(j) | Bytecode::BrFalse(j) if *j <= i => {
                let (cur_loop, _last_continue) = loop_stack.last().unwrap();
                let is_continue = *j <= i;
                if is_continue && j != cur_loop {
                    // Invalid back jump. Cannot back jump outside of the current loop
                    Err(err_at_offset(StatusCode::INVALID_LOOP_CONTINUE, i as usize))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    })
}

fn check_breaks(context: &ControlFlowVerifier, labels: &[Label]) -> VMResult<()> {
    check_code(context, labels, |loop_stack, i, instr| {
        match instr {
            // Forward jump/"break"
            Bytecode::Branch(j) | Bytecode::BrTrue(j) | Bytecode::BrFalse(j) if *j > i => {
                match loop_stack.last() {
                    Some((_cur_loop, last_continue))
                        if j > last_continue && *j != last_continue + 1 =>
                    {
                        // Invalid loop break. Must break immediately to the instruction after
                        // the last continue
                        Err(err_at_offset(StatusCode::INVALID_LOOP_BREAK, i as usize))
                    }
                    _ => Ok(()),
                }
            }
            _ => Ok(()),
        }
    })
}

fn check_no_loop_splits(context: &ControlFlowVerifier, labels: &[Label]) -> VMResult<()> {
    let is_break = |loop_stack: &Vec<(CodeOffset, CodeOffset)>, jump_target: CodeOffset| -> bool {
        match loop_stack.last() {
            None => false,
            Some((_cur_loop, last_continue)) => jump_target > *last_continue,
        }
    };
    let loop_depth = count_loop_depth(&labels);
    check_code(context, labels, |loop_stack, i, instr| {
        match instr {
            // Forward jump/"break"
            Bytecode::Branch(j) | Bytecode::BrTrue(j) | Bytecode::BrFalse(j)
                if *j > i && !is_break(loop_stack, *j) =>
            {
                let j = *j;
                let before_depth = loop_depth[i as usize];
                let after_depth = match &labels[j as usize] {
                    Label::Loop { .. } => loop_depth[j as usize] - 1,
                    Label::Code => loop_depth[j as usize],
                };
                if before_depth != after_depth {
                    // Invalid forward jump. Entered the middle of a loop
                    Err(err_at_offset(StatusCode::INVALID_LOOP_SPLIT, i as usize))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    })
}

// Only called after continues are verified, so we can assume that loops are well nested
fn count_loop_depth(labels: &[Label]) -> Vec<usize> {
    let last_continues: HashSet<CodeOffset> = labels
        .iter()
        .filter_map(|label| match label {
            Label::Loop { last_continue } => Some(*last_continue),
            Label::Code => None,
        })
        .collect();
    let mut count = 0;
    let mut counts = vec![];
    for (idx, label) in labels.iter().enumerate() {
        if let Label::Loop { .. } = label {
            count += 1
        }
        counts.push(count);
        if last_continues.contains(&idx.try_into().unwrap()) {
            count -= 1;
        }
    }
    counts
}
