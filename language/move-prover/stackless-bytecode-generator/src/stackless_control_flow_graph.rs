// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Adapted from control_flow_graph for Bytecode, this module defines the control-flow graph on
//! Stackless Bytecode used in analysis as part of Move prover.

use crate::stackless_bytecode::{Bytecode, Label};
use bytecode_verifier::control_flow_graph::{BlockId, ControlFlowGraph};
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::CodeOffset;

type Map<K, V> = BTreeMap<K, V>;
type Set<V> = BTreeSet<V>;

struct BasicBlock {
    entry: CodeOffset,
    exit: CodeOffset,
    successors: Vec<BlockId>,
    predecessors: Vec<BlockId>,
}

pub struct StacklessControlFlowGraph {
    blocks: Map<BlockId, BasicBlock>,
}

const ENTRY_BLOCK_ID: BlockId = 0;

impl StacklessControlFlowGraph {
    pub fn new(code: &[Bytecode]) -> Self {
        let label_offsets = Bytecode::label_offsets(code);
        // First go through and collect block ids, i.e., offsets that begin basic blocks.
        // Need to do this first in order to handle backwards edges.
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

        // Create basic blocks
        let mut cfg = StacklessControlFlowGraph { blocks: Map::new() };
        let mut entry = 0;
        for pc in 0..code.len() {
            let co_pc: CodeOffset = pc as CodeOffset;

            // Create a basic block
            if StacklessControlFlowGraph::is_end_of_block(co_pc, code, &block_ids) {
                let successors = Bytecode::get_successors(co_pc, code, &label_offsets);
                let bb = BasicBlock {
                    entry,
                    exit: co_pc,
                    successors,
                    predecessors: vec![],
                };
                cfg.blocks.insert(entry, bb);
                entry = co_pc + 1;
            }
        }
        assert_eq!(entry, code.len() as CodeOffset);

        // Create predecessor edges.
        for block_id in cfg.blocks.keys().cloned().collect_vec() {
            let successors = cfg.blocks.get(&block_id).unwrap().successors.clone();
            for succ in successors {
                let predecessors = &mut cfg.blocks.get_mut(&succ).unwrap().predecessors;
                if !predecessors.contains(&block_id) {
                    predecessors.push(block_id);
                }
            }
        }

        cfg
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

        if let Some(label) = bytecode.branch_dest() {
            block_ids.insert(*label_offsets.get(&label).unwrap());
        }

        if bytecode.is_branch() && pc + 1 < (code.len() as CodeOffset) {
            block_ids.insert(pc + 1);
        }
    }
}

impl ControlFlowGraph for StacklessControlFlowGraph {
    fn block_start(&self, block_id: BlockId) -> CodeOffset {
        self.blocks[&block_id].entry
    }

    fn block_end(&self, block_id: BlockId) -> CodeOffset {
        self.blocks[&block_id].exit
    }

    fn successors(&self, block_id: BlockId) -> &Vec<BlockId> {
        &self.blocks[&block_id].successors
    }

    fn blocks(&self) -> Vec<BlockId> {
        self.blocks.keys().cloned().collect()
    }

    fn instr_indexes(&self, block_id: BlockId) -> Box<dyn Iterator<Item = CodeOffset>> {
        Box::new(self.block_start(block_id)..=self.block_end(block_id))
    }

    fn num_blocks(&self) -> u16 {
        self.blocks.len() as u16
    }

    fn entry_block_id(&self) -> BlockId {
        ENTRY_BLOCK_ID
    }
}
