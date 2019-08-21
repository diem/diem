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

/// Reversed view of control flow graph to support backward analysis
pub struct BackwardControlFlowGraph {
    forward_cfg: VMControlFlowGraph,
    exit_block_id: BlockId,
    predecessors: Map<BlockId, Vec<BlockId>>,
}

impl BackwardControlFlowGraph {
    pub fn new(code: &[Bytecode]) -> Self {
        let forward_cfg = VMControlFlowGraph::new(code);
        // create a single exit block to use as the entry block in the trait
        let exit_block_id = forward_cfg.num_blocks() + 1;
        println!("Created CFG with exit block {:?}", exit_block_id);
        let mut predecessors = Map::new();
        // create a predecessor map. the predecessors of the entry block are any blocks with no
        // successors
        for (block_id, block) in &forward_cfg.blocks {
            if !predecessors.contains_key(block_id) {
                predecessors.insert(*block_id, Vec::new());
            }

            let successors = &block.successors;
            if successors.is_empty() {
                predecessors
                    .entry(exit_block_id)
                    .or_insert_with(Vec::new)
                    .push(*block_id)
            } else {
                for successor_id in successors {
                    predecessors
                        .entry(*successor_id)
                        .or_insert_with(Vec::new)
                        .push(*block_id)
                }
            }
        }

        Self {
            forward_cfg,
            exit_block_id,
            predecessors,
        }
    }

    pub fn is_exit_block(&self, block_id: BlockId) -> bool {
        self.exit_block_id == block_id
    }
}

impl ControlFlowGraph for BackwardControlFlowGraph {
    // Like normal CFG, but everything is backward

    fn block_start(&self, block_id: &BlockId) -> CodeOffset {
        if self.is_exit_block(*block_id) {
            *block_id
        } else {
            self.forward_cfg.blocks[block_id].exit
        }
    }

    fn block_end(&self, block_id: &BlockId) -> CodeOffset {
        if self.is_exit_block(*block_id) {
            *block_id
        } else {
            self.forward_cfg.blocks[block_id].entry
        }
    }

    fn successors(&self, block_id: &BlockId) -> &Vec<BlockId> {
        let res = &self.predecessors[block_id];
        println!("got {:?} successors for {:?}", res.len(), block_id);
        res
    }

    fn instr_indexes(&self, block_id: &BlockId) -> Box<dyn Iterator<Item = CodeOffset>> {
      Box::new((self.block_end(block_id)..=self.block_start(block_id)).rev())
    }

    fn blocks(&self) -> Vec<BlockId> {
        let mut blocks = self.forward_cfg.blocks();
        blocks.push(self.exit_block_id);
        blocks
    }

    fn num_blocks(&self) -> u16 {
        self.forward_cfg.num_blocks() + 1
    }

    fn entry_block_id(&self) -> BlockId {
        self.exit_block_id
    }
}
