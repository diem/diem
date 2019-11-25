// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::abstract_state::{AbstractValue, BorrowState};
use rand::{rngs::StdRng, Rng};
use std::collections::{HashMap, VecDeque};
use vm::file_format::{Bytecode, FunctionSignature, Kind, SignatureToken};

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
        signature: &FunctionSignature,
        target_blocks: BlockIDSize,
    ) -> CFG {
        checked_precondition!(target_blocks > 0, "The CFG must haave at least one block");
        let mut basic_blocks: HashMap<BlockIDSize, BasicBlock> = HashMap::new();
        // Generate basic blocks
        for i in 0..target_blocks {
            basic_blocks.insert(i, BasicBlock::new());
        }
        // Generate control flow edges
        let mut edges: Vec<(BlockIDSize, BlockIDSize)> = Vec::new();
        let mut block_queue: VecDeque<BlockIDSize> = VecDeque::new();
        let mut current_block_id = 0;

        block_queue.push_back(current_block_id);
        current_block_id += 1;

        while current_block_id < target_blocks && !block_queue.is_empty() {
            let front_block = block_queue.pop_front();
            // `front_block` will be `Some` because the block queue is not empty
            assume!(front_block.is_some());
            let parent_block_id = front_block.unwrap();
            // The number of edges will be at most `2*target_blocks``
            // Since target blocks is at most a `u16`, this will not overflow even if
            // `usize` is a `u32`
            assume!(edges.len() < usize::max_value());
            edges.push((parent_block_id, current_block_id));
            block_queue.push_back(current_block_id);
            // `current_block_id` is bound by the max og `target_block_size`
            verify!(current_block_id < u16::max_value());
            current_block_id += 1;
            // Generate a second child edge with prob = 1/2
            if rng.gen_bool(0.5) && current_block_id < target_blocks {
                // The number of edges will be at most `2*target_blocks``
                // Since target blocks is at most a `u16`, this will not overflow even if
                // `usize` is a `u32`
                verify!(edges.len() < usize::max_value());
                edges.push((parent_block_id, current_block_id));
                block_queue.push_back(current_block_id);
                // `current_block_id` is bound by the max og `target_block_size`
                verify!(current_block_id < u16::max_value());
                current_block_id += 1;
            }
        }

        // Connect remaining blocks to return
        while !block_queue.is_empty() {
            let front_block = block_queue.pop_front();
            // `front_block` will be `Some` because the block queue is not empty
            assume!(front_block.is_some());
            let parent_block_id = front_block.unwrap();
            // By the precondition of the function
            assume!(target_blocks > 0);
            if parent_block_id != target_blocks - 1 {
                edges.push((parent_block_id, target_blocks - 1));
            }
        }
        debug!("Edges: {:?}", edges);

        // Build the CFG
        let mut cfg = CFG {
            basic_blocks,
            edges,
        };
        // Assign locals to basic blocks
        assume!(target_blocks == 0 || !cfg.basic_blocks.is_empty());
        CFG::add_locals(&mut cfg, &mut rng, locals, signature.arg_types.len());
        cfg
    }

    /// Get a reference to all of the basic blocks of the CFG
    pub fn get_basic_blocks(&self) -> &HashMap<BlockIDSize, BasicBlock> {
        &self.basic_blocks
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
                // Length is bound by iteration on `self.edges`
                verify!(children_ids.len() < usize::max_value());
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
                // Iteration is bound by the self.edges vector length
                verify!(parent_ids.len() < usize::max_value());
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
        let first_basic_block = self.basic_blocks.get(&block_ids[0]);
        // Implication of preconditon
        assume!(first_basic_block.is_some());
        let first_basic_block_locals_out = &first_basic_block.unwrap().locals_out;
        let locals_len = first_basic_block_locals_out.len();
        let mut locals_out = BlockLocals::new();
        for local_index in 0..locals_len {
            let abstract_value = first_basic_block_locals_out[&local_index].0.clone();
            let mut availability = BorrowState::Available;
            for block_id in block_ids.iter() {
                // A local is available for a block if it is available in every
                // parent's outgoing locals
                let basic_block = self.basic_blocks.get(block_id);
                // Every block ID in the sequence should be valid
                assume!(basic_block.is_some());
                if basic_block.unwrap().locals_out[&local_index].1 == BorrowState::Unavailable {
                    availability = BorrowState::Unavailable;
                }
            }
            locals_out.insert(local_index, (abstract_value, availability));
        }
        locals_out
    }

    /// Randomly vary the availability of locals
    fn vary_locals(rng: &mut StdRng, locals: BlockLocals) -> BlockLocals {
        let mut locals = locals;
        for (_, (abstr_val, availability)) in locals.iter_mut() {
            if rng.gen_bool(0.5) {
                if *availability == BorrowState::Available {
                    *availability = BorrowState::Unavailable;
                } else {
                    *availability = BorrowState::Available;
                }
            }

            if abstr_val.is_generic() {
                *availability = BorrowState::Unavailable;
            }
        }
        locals
    }

    /// Add the incoming and outgoing locals for each basic block in the control flow graph.
    /// Currently the incoming and outgoing locals are the same for each block.
    fn add_locals(cfg: &mut CFG, mut rng: &mut StdRng, locals: &[SignatureToken], args_len: usize) {
        precondition!(
            !cfg.basic_blocks.is_empty(),
            "Cannot add locals to empty cfg"
        );
        debug!("add locals: {:#?}", locals);
        for block_id in 0..cfg.basic_blocks.len() {
            let cfg_copy = cfg.clone();
            let basic_block = cfg
                .basic_blocks
                .get_mut(&(block_id as BlockIDSize))
                .unwrap();
            if cfg_copy.num_parents(block_id as BlockIDSize) == 0 {
                basic_block.locals_in = locals
                    .iter()
                    .enumerate()
                    .map(|(i, token)| {
                        let borrow_state = if i < args_len {
                            BorrowState::Available
                        } else {
                            BorrowState::Unavailable
                        };
                        (
                            i,
                            (
                                AbstractValue::new_value(token.clone(), Kind::Unrestricted),
                                borrow_state,
                            ),
                        )
                    })
                    .collect();
            } else {
                // Implication of precondition
                assume!(!cfg_copy.basic_blocks.is_empty());
                basic_block.locals_in =
                    cfg_copy.merge_locals(cfg_copy.get_parent_ids(block_id as BlockIDSize));
            }
            basic_block.locals_out = CFG::vary_locals(&mut rng, basic_block.locals_in.clone());
        }
    }

    /// Decide the serialization order of the blocks in the CFG
    pub fn serialize_block_order(&self) -> Vec<BlockIDSize> {
        let mut block_order: Vec<BlockIDSize> = Vec::new();
        let mut block_queue: VecDeque<BlockIDSize> = VecDeque::new();
        block_queue.push_back(0);
        while !block_queue.is_empty() {
            let block_id_front = block_queue.pop_front();
            // The queue is non-empty so the front block id will not be none
            assume!(block_id_front.is_some());
            let block_id = block_id_front.unwrap();
            let child_ids = self.get_children_ids(block_id);
            if child_ids.len() == 2 {
                block_queue.push_front(child_ids[0]);
                block_queue.push_back(child_ids[1]);
            } else if child_ids.len() == 1 {
                block_queue.push_back(child_ids[0]);
            } else if !child_ids.is_empty() {
                // We construct the CFG such that blocks have either 0, 1, or 2
                // children.
                unreachable!(
                    "Invalid number of children for basic block {:?}",
                    child_ids.len()
                );
            }
            // This operation is expensive but is performed just when
            // serializing the module.
            if !block_order.contains(&block_id) {
                block_order.push(block_id);
            }
        }
        debug!("Block order: {:?}", block_order);
        block_order
    }

    /// Get the serialized code offset of a basic block based on its position in the serialized
    /// instruction sequence.
    fn get_block_offset(cfg: &CFG, block_order: &[BlockIDSize], block_id: BlockIDSize) -> u16 {
        checked_assume!(
            (0..block_id).all(|id| cfg.basic_blocks.get(&id).is_some()),
            "Error: Invalid block_id given"
        );
        let mut offset: u16 = 0;
        for i in block_order {
            if *i == block_id {
                break;
            }
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
        let block_order = self.serialize_block_order();
        for block_id in &block_order {
            let block = self.basic_blocks.get_mut(&block_id);
            // The generated block order contains every block
            assume!(block.is_some());
            let block = block.unwrap();
            // All basic blocks should have instructions filled in at this point
            checked_assume!(
                !block.instructions.is_empty(),
                "Error: block created with no instructions",
            );
            let last_instruction_index = block.instructions.len() - 1;
            let child_ids = cfg_copy.get_children_ids(*block_id);
            if child_ids.len() == 2 {
                // The left child (fallthrough) is serialized before the right (jump)
                let offset = CFG::get_block_offset(&cfg_copy, &block_order, child_ids[1]);
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
            } else if child_ids.len() == 1 {
                let offset = CFG::get_block_offset(&cfg_copy, &block_order, child_ids[0]);
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
