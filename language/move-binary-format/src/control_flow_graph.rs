// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the control-flow graph uses for bytecode verification.
use crate::file_format::{Bytecode, CodeOffset};
use std::collections::{BTreeMap, BTreeSet};

// BTree/Hash agnostic type wrappers
type Map<K, V> = BTreeMap<K, V>;
type Set<V> = BTreeSet<V>;

pub type BlockId = CodeOffset;

/// A trait that specifies the basic requirements for a CFG
pub trait ControlFlowGraph {
    /// Start index of the block ID in the bytecode vector
    fn block_start(&self, block_id: BlockId) -> CodeOffset;

    /// End index of the block ID in the bytecode vector
    fn block_end(&self, block_id: BlockId) -> CodeOffset;

    /// Successors of the block ID in the bytecode vector
    fn successors(&self, block_id: BlockId) -> &Vec<BlockId>;

    fn next_block(&self, block_id: BlockId) -> Option<CodeOffset>;

    /// Iterator over the indexes of instructions in this block
    fn instr_indexes(&self, block_id: BlockId) -> Box<dyn Iterator<Item = CodeOffset>>;

    /// Return an iterator over the blocks of the CFG
    fn blocks(&self) -> Vec<BlockId>;

    /// Return the number of blocks (vertices) in the control flow graph
    fn num_blocks(&self) -> u16;

    /// Return the id of the entry block for this control-flow graph
    /// Note: even a CFG with no instructions has an (empty) entry block.
    fn entry_block_id(&self) -> BlockId;

    /// Map from blocks with a loop's last continue to that loop's starts
    fn loop_last_continue_blocks(&self) -> &Map<BlockId, BlockId>;
}

struct BasicBlock {
    exit: CodeOffset,
    successors: Vec<BlockId>,
}

/// The control flow graph that we build from the bytecode.
pub struct VMControlFlowGraph {
    /// The basic blocks
    blocks: Map<BlockId, BasicBlock>,
    /// Basic block ordering for traversal
    traversal_successors: Map<BlockId, BlockId>,
    /// Map from blocks with a loop's last continue to that loop's starts
    loop_last_continue_blocks: Map<BlockId, BlockId>,
}

pub struct LoopBounds {
    pub loop_start: CodeOffset,
    pub last_continue: CodeOffset,
}

impl BasicBlock {
    pub fn display(&self, entry: BlockId) {
        println!("+=======================+");
        println!("| Enter:  {}            |", entry);
        println!("+-----------------------+");
        println!("==> Children: {:?}", self.successors);
        println!("+-----------------------+");
        println!("| Exit:   {}            |", self.exit);
        println!("+=======================+");
    }
}

const ENTRY_BLOCK_ID: BlockId = 0;

impl VMControlFlowGraph {
    pub fn new(loop_bounds: Vec<LoopBounds>, code: &[Bytecode]) -> Self {
        let code_len = code.len() as CodeOffset;
        // First go through and collect block ids, i.e., offsets that begin basic blocks.
        // Need to do this first in order to handle backwards edges.
        let mut block_ids = Set::new();
        block_ids.insert(ENTRY_BLOCK_ID);
        for pc in 0..code.len() {
            VMControlFlowGraph::record_block_ids(pc as CodeOffset, code, &mut block_ids);
        }

        // Create basic blocks
        let mut blocks = Map::new();
        let mut entry = 0;
        let mut exit_to_entry = Map::new();
        for pc in 0..code.len() {
            let co_pc = pc as CodeOffset;

            // Create a basic block
            if Self::is_end_of_block(co_pc, code, &block_ids) {
                let exit = co_pc;
                exit_to_entry.insert(exit, entry);
                let successors = Bytecode::get_successors(co_pc, code);
                let bb = BasicBlock { exit, successors };
                blocks.insert(entry, bb);
                entry = co_pc + 1;
            }
        }
        let blocks = blocks;
        assert_eq!(entry, code_len);

        // Determine traversal order
        // build a DAG subgraph (remove the loop back edges)
        let dag: Map<BlockId, Set<BlockId>> = blocks
            .iter()
            .map(|(id, block)| {
                let id = *id;
                let non_loop_continue_successors = block
                    .successors
                    .iter()
                    // remove the loop back edges
                    .filter(|successor| **successor > id)
                    .copied()
                    .collect();
                (id, non_loop_continue_successors)
            })
            .collect();

        // build the post-order traversal
        let mut post_order = Vec::with_capacity(blocks.len());
        let mut finished = Set::new();
        let mut stack = vec![(ENTRY_BLOCK_ID, /* is_first_visit */ true)];
        while let Some((cur, is_first_visit)) = stack.pop() {
            if is_first_visit {
                stack.push((cur, false));
                stack.extend(
                    dag[&cur]
                        .iter()
                        .filter(|successor| !finished.contains(*successor))
                        .map(|successor| (*successor, /* is_first_visit */ true)),
                );
            } else {
                debug_assert!(dag[&cur]
                    .iter()
                    .all(|successor| finished.contains(successor)));
                if finished.insert(cur) {
                    post_order.push(cur)
                }
            }
        }
        // traversal order is the reverse post-order
        let traversal_order = {
            post_order.reverse();
            post_order
        };
        // build a mapping from a block id to the next block id in the traversal order
        let traversal_successors = traversal_order
            .windows(2)
            .map(|window| {
                debug_assert!(window.len() == 2);
                (window[0], window[1])
            })
            .collect();

        // Determine loop ends
        let mut loop_last_continue_blocks: Map<BlockId, BlockId> = Map::new();
        for LoopBounds {
            loop_start,
            last_continue,
        } in loop_bounds
        {
            let continue_block_start = exit_to_entry[&last_continue];
            debug_assert!(blocks.contains_key(&loop_start));
            debug_assert!(blocks.contains_key(&continue_block_start));
            debug_assert!(!loop_last_continue_blocks.contains_key(&continue_block_start));
            loop_last_continue_blocks.insert(continue_block_start, loop_start);
        }

        VMControlFlowGraph {
            blocks,
            traversal_successors,
            loop_last_continue_blocks,
        }
    }

    pub fn display(&self) {
        for (entry, block) in &self.blocks {
            block.display(*entry);
        }
        println!("Traversal: {:#?}", self.traversal_successors);
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
            let successors = self.successors(block_id);
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

    fn block_start(&self, block_id: BlockId) -> CodeOffset {
        block_id
    }

    fn block_end(&self, block_id: BlockId) -> CodeOffset {
        self.blocks[&block_id].exit
    }

    fn successors(&self, block_id: BlockId) -> &Vec<BlockId> {
        &self.blocks[&block_id].successors
    }

    fn next_block(&self, block_id: BlockId) -> Option<CodeOffset> {
        debug_assert!(self.blocks.contains_key(&block_id));
        self.traversal_successors.get(&block_id).copied()
    }

    fn instr_indexes(&self, block_id: BlockId) -> Box<dyn Iterator<Item = CodeOffset>> {
        Box::new(self.block_start(block_id)..=self.block_end(block_id))
    }

    fn blocks(&self) -> Vec<BlockId> {
        self.blocks.keys().cloned().collect()
    }

    fn num_blocks(&self) -> u16 {
        self.blocks.len() as u16
    }

    fn entry_block_id(&self) -> BlockId {
        ENTRY_BLOCK_ID
    }

    fn loop_last_continue_blocks(&self) -> &Map<BlockId, BlockId> {
        &self.loop_last_continue_blocks
    }
}
