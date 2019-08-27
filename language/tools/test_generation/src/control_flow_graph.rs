// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::abstract_state::{AbstractValue, BorrowState};
use rand::{rngs::StdRng, Rng};
use std::collections::HashMap;
use vm::file_format::{Bytecode, FunctionSignature, SignatureToken};

/// This type holds basic block identifiers
type BlockIDSize = u16;

/// This type represents the locals that a basic block has
type BlockLocals = HashMap<usize, (AbstractValue, BorrowState)>;

/// This represents a basic block in a control flow graph
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// The starting locals
    locals_in: BlockLocals,

    /// The locals at the end of the block
    locals_out: BlockLocals,

    /// The instructions that comprise the block
    instructions: Vec<Bytecode>,
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        BasicBlock {
            locals_in: HashMap::new(),
            locals_out: HashMap::new(),
            instructions: Vec::new(),
        }
    }

    /// Get the locals coming into the block
    pub fn get_locals_in(&self) -> &BlockLocals {
        &self.locals_in
    }

    /// Get the locals going out of the block
    pub fn get_locals_out(&self) -> &BlockLocals {
        &self.locals_out
    }

    /// Set the list of instructions that comprise the block
    pub fn set_instructions(&mut self, instructions: Vec<Bytecode>) {
        self.instructions = instructions
    }
}

/// A control flow graph
#[derive(Debug, Clone)]
pub struct CFG {
    /// The set of basic blocks that make up the graph, mapped to `BlockIDSize`'s used
    /// as their identifiers
    basic_blocks: HashMap<BlockIDSize, BasicBlock>,

    /// The directed edges of the graph represented by pairs of basic block identifiers
    edges: Vec<(BlockIDSize, BlockIDSize)>,
}

impl CFG {
    /// Construct a control flow graph that contains empty basic blocks with set incoming
    /// and outgoing locals.
    /// Currently the control flow graph is acyclic.
    pub fn new(
        mut rng: &mut StdRng,
        locals: &[SignatureToken],
        _signature: &FunctionSignature,
        target_blocks: BlockIDSize,
    ) -> CFG {
        let mut basic_blocks: HashMap<BlockIDSize, BasicBlock> = HashMap::new();
        // Generate basic blocks
        for i in 0..target_blocks {
            basic_blocks.insert(i, BasicBlock::new());
        }
        // Generate control flow edges
        let mut edges: Vec<(BlockIDSize, BlockIDSize)> = Vec::new();
        for i in 0..target_blocks {
            let child_1 = rng.gen_range(i, target_blocks);
            edges.push((i, child_1));
            // At most two children per block
            if rng.gen_range(0, 1) == 1 {
                let child_2 = rng.gen_range(i, target_blocks);
                if child_2 != child_1 {
                    edges.push((i, child_2));
                }
            }
        }
        // Build the CFG
        let mut cfg = CFG {
            basic_blocks,
            edges,
        };
        // Assign locals to basic blocks
        CFG::add_locals(&mut cfg, &mut rng, locals);
        cfg
    }

    /// Get a mutable reference to all of the basic blocks of the CFG
    pub fn get_basic_blocks_mut(&mut self) -> &mut HashMap<BlockIDSize, BasicBlock> {
        &mut self.basic_blocks
    }

    /// Retrieve the block IDs of all children of the given basic block `block_id`
    pub fn get_children_ids(&self, block_id: BlockIDSize) -> Vec<BlockIDSize> {
        let mut children_ids: Vec<BlockIDSize> = Vec::new();
        for (parent, child) in self.edges.iter() {
            if *parent == block_id {
                children_ids.push(*child);
            }
        }
        children_ids
    }

    /// Retrieve the number of children the given basic block `block_id`
    pub fn num_children(&self, block_id: BlockIDSize) -> u8 {
        // A `u8` is sufficient; blocks will have at most two children
        self.get_children_ids(block_id).len() as u8
    }

    /// Retrieve the block IDs of all parents of the given basic block `block_id`
    pub fn get_parent_ids(&self, block_id: BlockIDSize) -> Vec<BlockIDSize> {
        let mut parent_ids: Vec<BlockIDSize> = Vec::new();
        for (parent, child) in self.edges.iter() {
            if *child == block_id {
                parent_ids.push(*parent);
            }
        }
        parent_ids
    }

    /// Retrieve the number of parents the given basic block `block_id`
    pub fn num_parents(&self, block_id: BlockIDSize) -> u8 {
        // A `u8` is sufficient; blocks will have at most two children
        self.get_parent_ids(block_id).len() as u8
    }

    /// Merge the outgoing locals of a set of blocks
    fn merge_locals(&self, block_ids: Vec<BlockIDSize>) -> BlockLocals {
        checked_precondition!(
            !block_ids.is_empty(),
            "Cannot merge locals of empty block list"
        );
        let mut locals_out = BlockLocals::new();
        let locals_len = self
            .basic_blocks
            .get(&block_ids[0])
            .unwrap()
            .locals_out
            .len();
        for local_index in 0..locals_len {
            let abstract_value = self.basic_blocks.get(&block_ids[0]).unwrap().locals_out
                [&local_index]
                .0
                .clone();
            let mut availability = BorrowState::Available;
            for block_id in block_ids.iter() {
                // A local is available for a block if it is available in every
                // parent's outgoing locals
                if self.basic_blocks.get(block_id).unwrap().locals_out[&local_index].1
                    == BorrowState::Unavailable
                {
                    availability = BorrowState::Unavailable;
                }
            }
            locals_out.insert(local_index, (abstract_value, availability));
        }
        locals_out
    }

    /// Randomly vary the availability of locals
    fn vary_locals(rng: &mut StdRng, locals: BlockLocals) -> BlockLocals {
        let mut locals = locals.clone();
        for (_, (_, availability)) in locals.iter_mut() {
            if rng.gen_range(0, 1) == 0 {
                if *availability == BorrowState::Available {
                    *availability = BorrowState::Unavailable;
                } else {
                    *availability = BorrowState::Available;
                }
            }
        }
        locals
    }

    /// Add the incoming and outgoing locals for each basic block in the control flow graph.
    /// Currently the incoming and outgoing locals are the same for each block.
    fn add_locals(cfg: &mut CFG, mut rng: &mut StdRng, locals: &[SignatureToken]) {
        let cfg_copy = cfg.clone();
        for (block_id, basic_block) in cfg.basic_blocks.iter_mut() {
            if cfg_copy.num_parents(*block_id) == 0 {
                basic_block.locals_in = locals
                    .iter()
                    .enumerate()
                    .map(|(i, token)| {
                        (
                            i,
                            (
                                AbstractValue::new_primitive(token.clone()),
                                BorrowState::Available,
                            ),
                        )
                    })
                    .collect();
            } else {
                basic_block.locals_in = cfg_copy.merge_locals(cfg_copy.get_parent_ids(*block_id));
            }
            basic_block.locals_out = CFG::vary_locals(&mut rng, basic_block.locals_in.clone());
        }
    }

    /// Get the serialized code offset of a basic block based on its position in the serialized
    /// instruction sequence.
    fn get_block_offset(cfg: &CFG, block_id: BlockIDSize) -> u16 {
        checked_assume!(
            (0..block_id).all(|id| cfg.basic_blocks.get(&id).is_some()),
            "Error: Invalid block_id given"
        );
        let mut offset: u16 = 0;
        for i in 0..block_id {
            if let Some(block) = cfg.basic_blocks.get(&i) {
                offset += block.instructions.len() as u16;
            }
        }
        offset
    }

    /// Serialize the control flow graph into a sequence of instructions. Set the offsets of branch
    /// instructions appropriately.
    pub fn serialize(&mut self) -> Vec<Bytecode> {
        checked_precondition!(
            !self.basic_blocks.is_empty(),
            "Error: CFG has no basic blocks"
        );
        let cfg_copy = self.clone();
        let mut bytecode: Vec<Bytecode> = Vec::new();
        for i in 0..self.basic_blocks.len() {
            let block_id = i as BlockIDSize;
            let block = self.basic_blocks.get_mut(&block_id).unwrap();
            let last_instruction_index = block.instructions.len() - 1;
            if cfg_copy.num_children(block_id) == 2 {
                let child_id: BlockIDSize = cfg_copy.get_children_ids(block_id)[1];
                // The left child (fallthrough) is serialized before the right (jump)
                let offset = CFG::get_block_offset(&cfg_copy, child_id);
                match block.instructions.last() {
                    Some(Bytecode::BrTrue(_)) => {
                        block.instructions[last_instruction_index] =
                            Bytecode::BrTrue(offset as u16);
                    }
                    Some(Bytecode::BrFalse(_)) => {
                        block.instructions[last_instruction_index] =
                            Bytecode::BrFalse(offset as u16);
                    }
                    _ => unreachable!(
                        "Error: unsupported two target jump instruction, {:#?}",
                        block.instructions.last()
                    ),
                };
            } else if cfg_copy.num_children(block_id) == 1 {
                let child_id: BlockIDSize = cfg_copy.get_children_ids(block_id)[0];
                let offset = CFG::get_block_offset(&cfg_copy, child_id);
                match block.instructions.last() {
                    Some(Bytecode::Branch(_)) => {
                        block.instructions[last_instruction_index] = Bytecode::Branch(offset);
                    }
                    _ => unreachable!(
                        "Error: unsupported one target jump instruction, {:#?}",
                        block.instructions.last()
                    ),
                }
            }
            bytecode.extend(block.instructions.clone());
        }
        debug!("Final bytecode: {:#?}", bytecode);
        bytecode
    }
}
