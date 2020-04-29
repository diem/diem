// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Adapted from control_flow_graph for Bytecode, this module defines the control-flow graph on
//! Stackless Bytecode used in analysis as part of Move prover.

use crate::stackless_bytecode::{Bytecode, Label};
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

type Map<K, V> = BTreeMap<K, V>;
type Set<V> = BTreeSet<V>;
pub type BlockId = CodeOffset;

struct BasicBlock {
    lower: CodeOffset,
    upper: CodeOffset,
    successors: Vec<BlockId>,
}

pub struct StacklessControlFlowGraph {
    entry_block_ids: Vec<BlockId>,
    blocks: Map<BlockId, BasicBlock>,
}

const ENTRY_BLOCK_ID: BlockId = 0;

impl StacklessControlFlowGraph {
    pub fn new_forward(code: &[Bytecode]) -> Self {
        Self {
            entry_block_ids: vec![ENTRY_BLOCK_ID],
            blocks: Self::collect_blocks(code),
        }
    }

    pub fn new_backward(code: &[Bytecode]) -> Self {
        let blocks = Self::collect_blocks(code);
        let mut block_id_to_predecessors: Map<BlockId, Vec<BlockId>> =
            blocks.keys().map(|block_id| (*block_id, vec![])).collect();
        for (block_id, block) in &blocks {
            for succ_block_id in &block.successors {
                let predecessors = &mut block_id_to_predecessors.get_mut(&succ_block_id).unwrap();
                predecessors.push(*block_id);
            }
        }
        Self {
            entry_block_ids: blocks
                .iter()
                .map(|(block_id, block)| {
                    if code[block.upper as usize].is_return() {
                        Some(*block_id)
                    } else {
                        None
                    }
                })
                .flatten()
                .collect(),
            blocks: block_id_to_predecessors
                .into_iter()
                .map(|(block_id, predecessors)| {
                    (
                        block_id,
                        BasicBlock {
                            lower: blocks[&block_id].lower,
                            upper: blocks[&block_id].upper,
                            successors: predecessors,
                        },
                    )
                })
                .collect(),
        }
    }

    fn collect_blocks(code: &[Bytecode]) -> Map<BlockId, BasicBlock> {
        // First go through and collect block ids, i.e., offsets that begin basic blocks.
        // Need to do this first in order to handle backwards edges.
        let label_offsets = Bytecode::label_offsets(code);
        let mut block_ids = Set::new();
        block_ids.insert(ENTRY_BLOCK_ID);
        for pc in 0..code.len() {
            StacklessControlFlowGraph::record_block_ids(
                pc as CodeOffset,
                code,
                &mut block_ids,
                &label_offsets,
            );
        }
        // Now construct blocks
        let mut blocks = Map::new();
        let mut entry = 0;
        for pc in 0..code.len() {
            let co_pc: CodeOffset = pc as CodeOffset;
            // Create a basic block
            if StacklessControlFlowGraph::is_end_of_block(co_pc, code, &block_ids) {
                let successors = Bytecode::get_successors(co_pc, code, &label_offsets);
                let bb = BasicBlock {
                    lower: entry,
                    upper: co_pc,
                    successors,
                };
                blocks.insert(entry, bb);
                entry = co_pc + 1;
            }
        }
        assert_eq!(entry, code.len() as CodeOffset);
        blocks
    }

    fn is_end_of_block(pc: CodeOffset, code: &[Bytecode], block_ids: &Set<BlockId>) -> bool {
        pc + 1 == (code.len() as CodeOffset) || block_ids.contains(&(pc + 1))
    }

    fn record_block_ids(
        pc: CodeOffset,
        code: &[Bytecode],
        block_ids: &mut Set<BlockId>,
        label_offsets: &BTreeMap<Label, CodeOffset>,
    ) {
        let bytecode = &code[pc as usize];

        for label in bytecode.branch_dests() {
            block_ids.insert(*label_offsets.get(&label).unwrap());
        }

        if bytecode.is_branch() && pc + 1 < (code.len() as CodeOffset) {
            block_ids.insert(pc + 1);
        }
    }
}

impl StacklessControlFlowGraph {
    pub fn block_start(&self, block_id: BlockId) -> CodeOffset {
        self.blocks[&block_id].lower
    }

    pub fn block_end(&self, block_id: BlockId) -> CodeOffset {
        self.blocks[&block_id].upper
    }

    pub fn successors(&self, block_id: BlockId) -> &Vec<BlockId> {
        &self.blocks[&block_id].successors
    }

    pub fn blocks(&self) -> Vec<BlockId> {
        self.blocks.keys().cloned().collect()
    }

    pub fn entry_blocks(&self) -> Vec<BlockId> {
        self.entry_block_ids.clone()
    }

    pub fn instr_indexes(
        &self,
        block_id: BlockId,
    ) -> Box<dyn DoubleEndedIterator<Item = CodeOffset>> {
        Box::new(self.block_start(block_id)..=self.block_end(block_id))
    }

    pub fn num_blocks(&self) -> u16 {
        self.blocks.len() as u16
    }
}
