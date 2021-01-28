// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Adapted from control_flow_graph for Bytecode, this module defines the control-flow graph on
//! Stackless Bytecode used in analysis as part of Move prover.

use crate::stackless_bytecode::{Bytecode, Label};
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

type Map<K, V> = BTreeMap<K, V>;
type Set<V> = BTreeSet<V>;
pub type BlockId = CodeOffset;

struct Block {
    successors: Vec<BlockId>,
    content: BlockContent,
}

#[derive(Copy, Clone)]
pub enum BlockContent {
    Basic {
        lower: CodeOffset,
        upper: CodeOffset,
    },
    Dummy,
}

pub struct StacklessControlFlowGraph {
    entry_block_id: BlockId,
    blocks: Map<BlockId, Block>,
    backward: bool,
}

const DUMMY_ENTRACE: BlockId = 0;
const DUMMY_EXIT: BlockId = 1;

impl StacklessControlFlowGraph {
    pub fn new_forward(code: &[Bytecode]) -> Self {
        Self {
            entry_block_id: DUMMY_ENTRACE,
            blocks: Self::collect_blocks(code),
            backward: false,
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
            entry_block_id: DUMMY_EXIT,
            blocks: block_id_to_predecessors
                .into_iter()
                .map(|(block_id, predecessors)| {
                    (
                        block_id,
                        Block {
                            successors: predecessors,
                            content: blocks[&block_id].content,
                        },
                    )
                })
                .collect(),
            backward: true,
        }
    }

    fn collect_blocks(code: &[Bytecode]) -> Map<BlockId, Block> {
        // First go through and collect basic block offsets.
        // Need to do this first in order to handle backwards edges.
        let label_offsets = Bytecode::label_offsets(code);
        let mut bb_offsets = Set::new();
        bb_offsets.insert(0);
        for pc in 0..code.len() {
            StacklessControlFlowGraph::record_block_ids(
                pc as CodeOffset,
                code,
                &mut bb_offsets,
                &label_offsets,
            );
        }
        // Now construct blocks
        let mut blocks = Map::new();
        // Maps basic block entry offsets to their key in blocks
        let mut offset_to_key = Map::new();
        // Block counter starts at 2 because entry and exit will be block 0 and 1
        let mut bcounter = 2;
        let mut block_entry = 0;
        for pc in 0..code.len() {
            let co_pc: CodeOffset = pc as CodeOffset;
            // Create a basic block
            if StacklessControlFlowGraph::is_end_of_block(co_pc, code, &bb_offsets) {
                let mut successors = Bytecode::get_successors(co_pc, code, &label_offsets);
                for successor in successors.iter_mut() {
                    *successor = *offset_to_key.entry(*successor).or_insert(bcounter);
                    bcounter = std::cmp::max(*successor + 1, bcounter);
                }
                if code[co_pc as usize].is_exit() {
                    successors.push(DUMMY_EXIT);
                }
                let bb = BlockContent::Basic {
                    lower: block_entry,
                    upper: co_pc,
                };
                let key = *offset_to_key.entry(block_entry).or_insert(bcounter);
                bcounter = std::cmp::max(key + 1, bcounter);
                blocks.insert(
                    key,
                    Block {
                        successors,
                        content: bb,
                    },
                );
                block_entry = co_pc + 1;
            }
        }
        assert_eq!(block_entry, code.len() as CodeOffset);
        let entry_bb = *offset_to_key.get(&0).unwrap();
        blocks.insert(
            DUMMY_ENTRACE,
            Block {
                successors: vec![entry_bb],
                content: BlockContent::Dummy,
            },
        );
        blocks.insert(
            DUMMY_EXIT,
            Block {
                successors: Vec::new(),
                content: BlockContent::Dummy,
            },
        );
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
    pub fn successors(&self, block_id: BlockId) -> &Vec<BlockId> {
        &self.blocks[&block_id].successors
    }

    pub fn content(&self, block_id: BlockId) -> &BlockContent {
        &self.blocks[&block_id].content
    }

    pub fn blocks(&self) -> Vec<BlockId> {
        self.blocks.keys().cloned().collect()
    }

    pub fn entry_block(&self) -> BlockId {
        self.entry_block_id
    }

    pub fn exit_block(&self) -> BlockId {
        if self.backward {
            DUMMY_ENTRACE
        } else {
            DUMMY_EXIT
        }
    }

    pub fn instr_indexes(
        &self,
        block_id: BlockId,
    ) -> Option<Box<dyn DoubleEndedIterator<Item = CodeOffset>>> {
        match self.blocks[&block_id].content {
            BlockContent::Basic { lower, upper } => Some(Box::new(lower..=upper)),
            BlockContent::Dummy => None,
        }
    }

    pub fn num_blocks(&self) -> u16 {
        self.blocks.len() as u16
    }

    pub fn is_dummmy(&self, block_id: BlockId) -> bool {
        matches!(self.blocks[&block_id].content, BlockContent::Dummy)
    }
}
