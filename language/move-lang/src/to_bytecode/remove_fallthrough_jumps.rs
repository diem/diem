// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::graph::Graph;
use move_ir_types::ast as IR;
use std::collections::{BTreeMap, BTreeSet, HashMap};

// Removes any "fall through jumps", i.e. this is a direct jump to the next instruction. However,
// if a fall-through jump is *the only instruction* in a loop header block, the jump is preserved.
// This is essential for loop invariant instrumentation performed by the Move prover. Without this
// preservation, nested loops might be coalesced to sharing a single loop header, e.g., in the case
// of `loop { loop { loop { ... } ... } ... }`, but we need these loops to have different headers
// such that we can instrument invariants for each loop respectively.
//
// The process of removing fall-through jumps and empty blocks repeats until a fixpoint is reached,
// as removing jumps might create empty blocks which could create more jumps to be cleaned up.

pub fn code(blocks: &mut IR::BytecodeBlocks) {
    let mut changed = true;
    while changed {
        let fall_through_removed = remove_fall_through(blocks);
        let block_removed = remove_empty_blocks(blocks);
        changed = fall_through_removed || block_removed;
    }
}

fn remove_fall_through(blocks: &mut IR::BytecodeBlocks) -> bool {
    use IR::Bytecode_ as B;

    let no_removals = collect_code_offsets_for_single_bytecode_loop_headers(blocks);
    let mut changed = false;
    let mut code_offset = 0;
    for idx in 0..(blocks.len() - 1) {
        let next_block = blocks.get(idx + 1).unwrap().0.clone();
        let (_, block) = blocks.get_mut(idx).unwrap();
        code_offset += block.len();
        let remove_last = matches!(
            &block.last().unwrap().value, B::Branch(lbl)
            if lbl == &next_block && !no_removals.contains(&(code_offset - 1))
        );
        if remove_last {
            changed = true;
            block.pop();
        }
    }
    changed
}

fn remove_empty_blocks(blocks: &mut IR::BytecodeBlocks) -> bool {
    let mut label_map = HashMap::new();
    let mut cur_label = None;
    let mut removed = false;
    let old_blocks = std::mem::replace(blocks, vec![]);
    for (label, block) in old_blocks.into_iter().rev() {
        if block.is_empty() {
            removed = true;
        } else {
            cur_label = Some(label.clone());
            blocks.push((label.clone(), block))
        }
        label_map.insert(label, cur_label.clone().unwrap());
    }
    blocks.reverse();

    if removed {
        super::remap_labels(blocks, &label_map);
    }

    removed
}

// control-flow graph construction
type CodeOffset = usize;

struct BasicBlock {
    entry: CodeOffset,
    exit: CodeOffset,
    successors: BTreeSet<CodeOffset>,
}

struct ControlFlowGraph {
    graph: Graph<CodeOffset>,
    blocks: BTreeMap<CodeOffset, BasicBlock>,
}

fn build_control_flow_graph(blocks: &[(IR::BlockLabel, IR::BytecodeBlock)]) -> ControlFlowGraph {
    use IR::Bytecode_ as B;

    // collect block ids
    let code_length: usize = blocks.iter().map(|(_, block)| block.len()).sum();
    let mut label_to_offset = BTreeMap::new();
    let mut block_ids = BTreeSet::new();
    let mut code_offset = 0;
    for (label, block) in blocks {
        label_to_offset.insert(label.clone(), code_offset);
        block_ids.insert(code_offset);
        for bytecode in block {
            match bytecode.value {
                B::Branch(_) | B::BrFalse(_) | B::BrTrue(_) | B::Ret | B::Abort => {
                    let next_offset = code_offset + 1;
                    if next_offset < code_length {
                        block_ids.insert(next_offset);
                    }
                }
                _ => {}
            }
            code_offset += 1;
        }
    }

    // construct blocks
    let mut cfg_blocks = BTreeMap::new();
    code_offset = 0;
    let mut entry = 0;
    for (_, block) in blocks {
        for bytecode in block {
            let next_offset = code_offset + 1;
            if next_offset == code_length || block_ids.contains(&next_offset) {
                // this is the end of the block
                let mut successors = BTreeSet::new();
                match &bytecode.value {
                    B::Ret | B::Abort => {
                        // terminator block, no successors
                    }
                    B::Branch(br_label) => {
                        // unconditional branch
                        successors.insert(*label_to_offset.get(br_label).unwrap());
                    }
                    B::BrTrue(br_label) | B::BrFalse(br_label) => {
                        // conditional branch, must have a fall through block
                        successors.insert(*label_to_offset.get(br_label).unwrap());
                        successors.insert(next_offset);
                    }
                    _ => {
                        // fall-through
                        successors.insert(next_offset);
                    }
                }
                let bb = BasicBlock {
                    entry,
                    exit: code_offset,
                    successors,
                };
                cfg_blocks.insert(entry, bb);
                entry = next_offset;
            }
            code_offset += 1;
        }
    }

    // construct the CFG
    let nodes = cfg_blocks.keys().copied().collect();
    let edges = cfg_blocks
        .values()
        .map(|bb| {
            bb.successors
                .iter()
                .map(move |successor| (bb.entry, *successor))
        })
        .flatten()
        .collect();

    ControlFlowGraph {
        graph: Graph::new(0, nodes, edges),
        blocks: cfg_blocks,
    }
}

fn collect_code_offsets_for_single_bytecode_loop_headers(
    blocks: &[(IR::BlockLabel, IR::BytecodeBlock)],
) -> BTreeSet<CodeOffset> {
    let cfg = build_control_flow_graph(blocks);
    let loops = cfg
        .graph
        .compute_reducible()
        .expect("Well-formed Move control-flow graph should be reducible");
    loops
        .into_iter()
        .map(|natural_loop| cfg.blocks.get(&natural_loop.loop_header).unwrap())
        .filter(|bb| bb.entry == bb.exit)
        .map(|bb| bb.entry)
        .collect()
}
