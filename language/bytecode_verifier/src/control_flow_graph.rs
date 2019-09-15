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
    /// Start index of the block ID in the bytecode vector
    fn block_start(&self, block_id: &BlockId) -> CodeOffset;

    /// End index of the block ID in the bytecode vector
    fn block_end(&self, block_id: &BlockId) -> CodeOffset;

    /// Successors of the block ID in the bytecode vector
    fn successors(&self, block_id: &BlockId) -> &Vec<BlockId>;

    /// Iterator over the indexes of instructions in this block
    fn instr_indexes(&self, block_id: &BlockId) -> Box<dyn Iterator<Item = CodeOffset>>;

    /// Return an iterator over the blocks of the CFG
    fn blocks(&self) -> Vec<BlockId>;

    /// Return the number of blocks (vertices) in the control flow graph
    fn num_blocks(&self) -> u16;

    /// Return the id of the entry block for this control-flow graph
    /// Note: even a CFG with no instructions has an (empty) entry block.
    fn entry_block_id(&self) -> BlockId;
}

struct BasicBlock {
    entry: CodeOffset,
    exit: CodeOffset,
    successors: Vec<BlockId>,
}

/// The control flow graph that we build from the bytecode.
pub struct VMControlFlowGraph {
    /// The basic blocks
    blocks: Map<BlockId, BasicBlock>,
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

const ENTRY_BLOCK_ID: BlockId = 0;

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

    /// A utility function that implements BFS-reachability from block_id with
    /// respect to get_targets function
    fn traverse_by(&self, block_id: BlockId) -> Vec<BlockId> {
        let mut ret = Vec::new();
        // We use this index to keep track of our frontier.
        let mut index = 0;
        // Guard against cycles
        let mut seen = Set::new();

        ret.push(block_id);
        seen.insert(&block_id);

        while index < ret.len() {
            let block_id = ret[index];
            index += 1;
            let successors = self.successors(&block_id);
            for block_id in successors.iter() {
                if !seen.contains(&block_id) {
                    ret.push(*block_id);
                    seen.insert(block_id);
                }
            }
        }

        ret
    }

    pub fn reachable_from(&self, block_id: BlockId) -> Vec<BlockId> {
        self.traverse_by(block_id)
    }
}

impl ControlFlowGraph for VMControlFlowGraph {
    // Note: in the following procedures, it's safe not to check bounds because:
    // - Every CFG (even one with no instructions) has a block at ENTRY_BLOCK_ID
    // - The only way to acquire new BlockId's is via block_successors()
    // - block_successors only() returns valid BlockId's
    // Note: it is still possible to get a BlockId from one CFG and use it in another CFG where it
    // is not valid. The design does not attempt to prevent this abuse of the API.

    fn block_start(&self, block_id: &BlockId) -> CodeOffset {
        self.blocks[block_id].entry
    }

    fn block_end(&self, block_id: &BlockId) -> CodeOffset {
        self.blocks[block_id].exit
    }

    fn successors(&self, block_id: &BlockId) -> &Vec<BlockId> {
        &self.blocks[block_id].successors
    }

    fn blocks(&self) -> Vec<BlockId> {
        self.blocks.keys().cloned().collect()
    }

    fn instr_indexes(&self, block_id: &BlockId) -> Box<dyn Iterator<Item = CodeOffset>> {
        Box::new(self.block_start(block_id)..=self.block_end(block_id))
    }

    fn num_blocks(&self) -> u16 {
        self.blocks.len() as u16
    }

    fn entry_block_id(&self) -> BlockId {
        ENTRY_BLOCK_ID
    }
}
