// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the control-flow graph uses for bytecode verification.
use std::{
    collections::{BTreeMap, BTreeSet},
    marker::Sized,
};
use vm::file_format::{Bytecode, CodeOffset};

// BTree/Hash agnostic type wrappers
type Map<K, V> = BTreeMap<K, V>;
type Set<V> = BTreeSet<V>;

pub type BlockId = CodeOffset;

/// A trait that specifies the basic requirements for a CFG
pub trait ControlFlowGraph: Sized {
    /// Given a vector of bytecodes, constructs the control flow graph for it.
    fn new(code: &[Bytecode]) -> Self;

    /// Given a block ID, return the reachable blocks from that block
    /// including the block itself.
    fn reachable_from(&self, block_id: BlockId) -> Vec<&BasicBlock>;

    /// Given an offset into the bytecode return the basic block ID that contains
    /// that offset
    fn block_id_of_offset(&self, code_offset: CodeOffset) -> Option<BlockId>;

    /// Given a block ID, return the corresponding basic block. Return None if
    /// the block ID is invalid.
    fn block_of_id(&self, block_id: BlockId) -> Option<&BasicBlock>;

    /// Return the number of blocks (vertices) in the control flow graph
    fn num_blocks(&self) -> u16;
}

/// A basic block
pub struct BasicBlock {
    /// Start index into bytecode vector
    pub entry: CodeOffset,

    /// End index into bytecode vector
    pub exit: CodeOffset,

    /// Flows-to
    pub successors: Set<BlockId>,
}

/// The control flow graph that we build from the bytecode.
pub struct VMControlFlowGraph {
    /// The basic blocks
    pub blocks: Map<BlockId, BasicBlock>,
}

impl BasicBlock {
    pub fn display(&self) {
        println!("+=======================+");
        println!("| Enter:  {}            |", self.entry);
        println!("+-----------------------+");
        println!("==> Children: {:?}", self.successors);
        println!("+-----------------------+");
        println!("| Exit:   {}            |", self.exit);
        println!("+=======================+");
    }
}

impl VMControlFlowGraph {
    pub fn display(&self) {
        for block in self.blocks.values() {
            block.display();
        }
    }

    fn is_end_of_block(pc: CodeOffset, code: &[Bytecode], block_ids: &Set<BlockId>) -> bool {
        pc + 1 == (code.len() as CodeOffset) || block_ids.contains(&(pc + 1))
    }

    fn record_block_ids(pc: CodeOffset, code: &[Bytecode], block_ids: &mut Set<BlockId>) {
        let bytecode = &code[pc as usize];

        if let Some(offset) = bytecode.offset() {
            block_ids.insert(*offset);
        }

        if bytecode.is_branch() && pc + 1 < (code.len() as CodeOffset) {
            block_ids.insert(pc + 1);
        }
    }

    /// A utility function that implements BFS-reachability from block_id with
    /// respect to get_targets function
    fn traverse_by(
        &self,
        get_targets: fn(&BasicBlock) -> &Set<BlockId>,
        block_id: BlockId,
    ) -> Vec<&BasicBlock> {
        let mut ret = Vec::new();
        // We use this index to keep track of our frontier.
        let mut index = 0;
        // Guard against cycles
        let mut seen = Set::new();

        let block = &self.blocks[&block_id];
        ret.push(block);
        seen.insert(&block_id);

        while index < ret.len() {
            let block = ret[index];
            index += 1;
            let successors = get_targets(&block);
            for block_id in successors.iter() {
                if !seen.contains(&block_id) {
                    ret.push(&self.blocks[&block_id]);
                    seen.insert(block_id);
                }
            }
        }

        ret
    }
}

impl ControlFlowGraph for VMControlFlowGraph {
    fn num_blocks(&self) -> u16 {
        self.blocks.len() as u16
    }

    fn block_id_of_offset(&self, code_offset: CodeOffset) -> Option<BlockId> {
        let mut index = None;

        for (block_id, block) in &self.blocks {
            if block.entry >= code_offset && block.exit <= code_offset {
                index = Some(*block_id);
            }
        }

        index
    }

    fn block_of_id(&self, block_id: BlockId) -> Option<&BasicBlock> {
        if self.blocks.contains_key(&block_id) {
            Some(&self.blocks[&block_id])
        } else {
            None
        }
    }

    fn reachable_from(&self, block_id: BlockId) -> Vec<&BasicBlock> {
        self.traverse_by(|block: &BasicBlock| &block.successors, block_id)
    }

    fn new(code: &[Bytecode]) -> Self {
        // First go through and collect block ids, i.e., offsets that begin basic blocks.
        // Need to do this first in order to handle backwards edges.
        let mut block_ids = Set::new();
        block_ids.insert(0);
        for pc in 0..code.len() {
            VMControlFlowGraph::record_block_ids(pc as CodeOffset, code, &mut block_ids);
        }

        // Create basic blocks
        let mut cfg = VMControlFlowGraph { blocks: Map::new() };
        let mut entry = 0;
        for pc in 0..code.len() {
            let co_pc: CodeOffset = pc as CodeOffset;

            // Create a basic block
            if VMControlFlowGraph::is_end_of_block(co_pc, code, &block_ids) {
                let successors = Bytecode::get_successors(co_pc, code);
                let bb = BasicBlock {
                    entry,
                    exit: co_pc,
                    successors,
                };
                cfg.blocks.insert(entry, bb);
                entry = co_pc + 1;
            }
        }

        assert!(entry == code.len() as CodeOffset);
        cfg
    }
}
