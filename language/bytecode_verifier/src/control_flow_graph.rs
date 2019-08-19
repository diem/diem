// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the control-flow graph uses for bytecode verification.
use std::collections::{BTreeMap, BTreeSet};
use vm::file_format::{Bytecode, CodeOffset};

// BTree/Hash agnostic type wrappers
type Map<K, V> = BTreeMap<K, V>;
type Set<V> = BTreeSet<V>;

pub type BlockId = CodeOffset;

/// A trait that specifies the basic requirements for a CFG
pub trait ControlFlowGraph {
    /// Given a block ID, return the corresponding basic block. Return None if
    /// the block ID is invalid.
    fn block_of_id(&self, block_id: BlockId) -> Option<&BasicBlock>;

    /// Return the number of blocks (vertices) in the control flow graph
    fn num_blocks(&self) -> u16;

    /// Return the id of the entry block.
    /// Note: even a CFG with no instructions has an (empty) entry block.
    fn entry_block_id(&self) -> BlockId;
}

/// A basic block
pub struct BasicBlock {
    /// Start index into bytecode vector
    pub entry: CodeOffset,

    /// End index into bytecode vector
    pub exit: CodeOffset,

    /// Flows-to
    pub successors: Vec<BlockId>,
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
    pub fn new(code: &[Bytecode]) -> Self {
        // First go through and collect block ids, i.e., offsets that begin basic blocks.
        // Need to do this first in order to handle backwards edges.
        let mut block_ids = Set::new();
        block_ids.insert(ENTRY_BLOCK_ID);
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

        assert_eq!(entry, code.len() as CodeOffset);
        cfg
    }

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
}

const ENTRY_BLOCK_ID: BlockId = 0;

impl ControlFlowGraph for VMControlFlowGraph {
    fn block_of_id(&self, block_id: BlockId) -> Option<&BasicBlock> {
        if self.blocks.contains_key(&block_id) {
            Some(&self.blocks[&block_id])
        } else {
            None
        }
    }

    fn num_blocks(&self) -> u16 {
        self.blocks.len() as u16
    }

    fn entry_block_id(&self) -> BlockId {
        ENTRY_BLOCK_ID
    }
}
