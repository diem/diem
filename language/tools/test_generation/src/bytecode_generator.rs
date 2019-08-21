// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    abstract_state::{AbstractState, BorrowState},
    common, summaries,
};
use rand::{rngs::StdRng, FromEntropy, Rng, SeedableRng};
use std::collections::HashMap;
use vm::file_format::{
    AddressPoolIndex, ByteArrayPoolIndex, Bytecode, FunctionSignature, SignatureToken,
    StringPoolIndex,
};

/// This type represents bytecode instructions that take a `u8`
type U8ToBytecode = fn(u8) -> Bytecode;

/// This type represents bytecode instructions that take a `u16`
type U16ToBytecode = fn(u16) -> Bytecode;

/// This type represents bytecode instructions that take a `u64`
type U64ToBytecode = fn(u64) -> Bytecode;

/// This type represents bytecode instructions that take a `StringPoolIndex`
type StringPoolIndexToBytecode = fn(StringPoolIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `AddressPoolIndex`
type AddressPoolIndexToBytecode = fn(AddressPoolIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `ByteArrayPoolIndex`
type ByteArrayPoolIndexToBytecode = fn(ByteArrayPoolIndex) -> Bytecode;

/// There are six types of bytecode instructions
#[derive(Debug, Clone)]
enum BytecodeType {
    /// Instructions that do not take an argument
    NoArg(Bytecode),

    /// Instructions that take a `u8`
    U8(U8ToBytecode),

    /// Instructions that take a `u16`
    U16(U16ToBytecode),

    /// Instructions that take a `u64`
    U64(U64ToBytecode),

    /// Instructions that take a `StringPoolIndex`
    StringPoolIndex(StringPoolIndexToBytecode),

    /// Instructions that take an `AddressPoolIndex`
    AddressPoolIndex(AddressPoolIndexToBytecode),

    /// Instructions that take a `ByteArrayPoolIndex`
    ByteArrayPoolIndex(ByteArrayPoolIndexToBytecode),
}

/// Abstraction for change to the stack size
#[derive(Debug, Copy, Clone, PartialEq)]
enum StackEffect {
    /// Represents an increase in stack size
    Add,

    /// Represents a decrease in stack size
    Sub,

    /// Represents no change in stack size
    Nop,
}

/// This datatype holds basic block identifiers
type BlockIDSize = u16;

/// This represents a basic block in a control flow graph
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// The starting locals
    locals_in: HashMap<usize, (SignatureToken, BorrowState)>,

    /// The locals at the end of the block
    locals_out: HashMap<usize, (SignatureToken, BorrowState)>,

    /// The instructions that comprise the block
    instructions: Vec<Bytecode>,
}

impl BasicBlock {
    fn new() -> BasicBlock {
        BasicBlock {
            locals_in: HashMap::new(),
            locals_out: HashMap::new(),
            instructions: Vec::new(),
        }
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
    /// Retrieve the block IDs of all children of the given basic block `block_id`
    fn get_children_ids(&self, block_id: BlockIDSize) -> Vec<BlockIDSize> {
        let mut children_ids: Vec<BlockIDSize> = Vec::new();
        for (parent, child) in self.edges.iter() {
            if *parent == block_id {
                children_ids.push(*child);
            }
        }
        children_ids
    }

    /// Retrieve the number of children the given basic block `block_id`
    fn num_children(&self, block_id: BlockIDSize) -> u8 {
        // A `u8` is sufficient; blocks will have at most two children
        self.get_children_ids(block_id).len() as u8
    }
}

/// Generates a sequence of bytecode instructions.
/// This generator has:
/// - `instructions`: A list of bytecode instructions to use for generation
/// - `rng`: A random number generator for uniform random choice of next instruction
#[derive(Debug, Clone)]
pub struct BytecodeGenerator {
    instructions: Vec<(StackEffect, BytecodeType)>,
    rng: StdRng,
}

impl BytecodeGenerator {
    /// The `BytecodeGenerator` is instantiated with a seed to use with
    /// its random number generator.
    pub fn new(seed: Option<[u8; 32]>) -> Self {
        let instructions: Vec<(StackEffect, BytecodeType)> = vec![
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Pop)),
            (StackEffect::Add, BytecodeType::U64(Bytecode::LdConst)),
            (
                StackEffect::Add,
                BytecodeType::StringPoolIndex(Bytecode::LdStr),
            ),
            (
                StackEffect::Add,
                BytecodeType::AddressPoolIndex(Bytecode::LdAddr),
            ),
            (StackEffect::Add, BytecodeType::NoArg(Bytecode::LdTrue)),
            (StackEffect::Add, BytecodeType::NoArg(Bytecode::LdFalse)),
            (
                StackEffect::Add,
                BytecodeType::ByteArrayPoolIndex(Bytecode::LdByteArray),
            ),
            (StackEffect::Add, BytecodeType::U8(Bytecode::CopyLoc)),
            (StackEffect::Add, BytecodeType::U8(Bytecode::MoveLoc)),
            (StackEffect::Sub, BytecodeType::U8(Bytecode::StLoc)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Add)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Sub)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Mul)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Div)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Mod)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::BitAnd)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::BitOr)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Xor)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Or)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::And)),
            (StackEffect::Nop, BytecodeType::NoArg(Bytecode::Not)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Eq)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Neq)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Lt)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Gt)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Le)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Ge)),
            (
                StackEffect::Add,
                BytecodeType::NoArg(Bytecode::GetTxnGasUnitPrice),
            ),
            (
                StackEffect::Add,
                BytecodeType::NoArg(Bytecode::GetTxnMaxGasUnits),
            ),
            (
                StackEffect::Add,
                BytecodeType::NoArg(Bytecode::GetGasRemaining),
            ),
            (
                StackEffect::Add,
                BytecodeType::NoArg(Bytecode::GetTxnSequenceNumber),
            ),
            (StackEffect::Nop, BytecodeType::U16(Bytecode::Branch)),
            (StackEffect::Sub, BytecodeType::U16(Bytecode::BrTrue)),
            (StackEffect::Sub, BytecodeType::U16(Bytecode::BrFalse)),
        ];
        let generator = match seed {
            Some(seed) => StdRng::from_seed(seed),
            None => StdRng::from_entropy(),
        };
        Self {
            instructions,
            rng: generator,
        }
    }

    /// Given an `AbstractState`, `state`, and a the number of locals the function has,
    /// this function returns a list of instructions whose preconditions are satisfied for
    /// the state.
    fn candidate_instructions(
        &mut self,
        state: AbstractState,
        locals_len: usize,
    ) -> Vec<(StackEffect, Bytecode)> {
        let mut matches: Vec<(StackEffect, Bytecode)> = Vec::new();
        let instructions = &self.instructions;
        for (stack_effect, instruction) in instructions.iter() {
            let instruction: Bytecode = match instruction {
                BytecodeType::NoArg(instruction) => instruction.clone(),
                BytecodeType::U8(instruction) => {
                    // Generate a random index into the locals
                    if locals_len > 0 {
                        let local_index: u8 = self.rng.gen_range(0, locals_len as u8);
                        instruction(local_index)
                    } else {
                        instruction(0)
                    }
                }
                BytecodeType::U16(instruction) => {
                    // Set 0 as the offset. This will be set correctly during serialization
                    instruction(0)
                }
                BytecodeType::U64(instruction) => {
                    // Generate a random u64 constant to load
                    let value = self.rng.gen_range(0, u64::max_value());
                    instruction(value)
                }
                BytecodeType::StringPoolIndex(instruction) => {
                    // TODO: Determine correct index
                    instruction(StringPoolIndex::new(0))
                }
                BytecodeType::AddressPoolIndex(instruction) => {
                    // TODO: Determine correct index
                    instruction(AddressPoolIndex::new(0))
                }
                BytecodeType::ByteArrayPoolIndex(instruction) => {
                    // TODO: Determine correct index
                    instruction(ByteArrayPoolIndex::new(0))
                }
            };
            let summary = summaries::instruction_summary(instruction.clone());
            let unsatisfied_preconditions = summary
                .preconditions
                .iter()
                .any(|precondition| !precondition(&state));
            if !unsatisfied_preconditions {
                matches.push((*stack_effect, instruction));
            }
        }
        matches
    }

    /// Select an instruction from the list of candidates based on the current state's
    /// stack size and the expected number of function return parameters.
    fn select_candidate(
        &mut self,
        return_len: usize,
        state: &AbstractState,
        candidates: &[(StackEffect, Bytecode)],
    ) -> Bytecode {
        let stack_len = state.stack_len();
        let prob_add = if stack_len > return_len {
            common::MUTATION_TOLERANCE / (stack_len as f32)
        } else {
            1.0
        };
        debug!("Pr[add] = {:?}", prob_add);
        let next_instruction_index;
        if self.rng.gen_range(0.0, 1.0) <= prob_add {
            let add_candidates: Vec<(StackEffect, Bytecode)> = candidates
                .iter()
                .filter(|(stack_effect, _)| {
                    *stack_effect == StackEffect::Add || *stack_effect == StackEffect::Nop
                })
                .cloned()
                .collect();
            debug!("Add candidates: [{:?}]", add_candidates);
            // Add candidates should not be empty unless the list of bytecode instructions is
            // changed
            if add_candidates.is_empty() {
                panic!("Could not find valid candidate");
            }
            next_instruction_index = self.rng.gen_range(0, add_candidates.len());
            add_candidates[next_instruction_index].1.clone()
        } else {
            let sub_candidates: Vec<(StackEffect, Bytecode)> = candidates
                .iter()
                .filter(|(stack_effect, _)| {
                    *stack_effect == StackEffect::Sub || *stack_effect == StackEffect::Nop
                })
                .cloned()
                .collect();
            debug!("Sub candidates: [{:?}]", sub_candidates);
            // Sub candidates should not be empty unless the list of bytecode instructions is
            // changed
            if sub_candidates.is_empty() {
                panic!("Could not find valid candidate");
            }
            next_instruction_index = self.rng.gen_range(0, sub_candidates.len());
            sub_candidates[next_instruction_index].1.clone()
        }
    }

    /// Transition an abstract state, `state` to the next state by applying all of the effects
    /// of a particular bytecode instruction, `instruction`.
    fn abstract_step(&self, state: AbstractState, instruction: Bytecode) -> AbstractState {
        summaries::instruction_summary(instruction)
            .effects
            .iter()
            .fold(state, |acc, effect| effect(&acc))
    }

    pub fn apply_instruction(
        &self,
        mut state: AbstractState,
        bytecode: &mut Vec<Bytecode>,
        instruction: Bytecode,
    ) -> AbstractState {
        debug!("**********************");
        debug!("Bytecode: [{:?}]", bytecode);
        debug!("State1: [{:?}]", state);
        debug!("Next instr: {:?}", instruction);
        state = self.abstract_step(state, instruction.clone());
        debug!("State2: {:?}", state);
        bytecode.push(instruction);
        debug!("**********************\n");
        state
    }

    /// Given a valid starting state `abstract_state_in`, generate a valid sequence of
    /// bytecode instructions such that `abstract_state_out` is reached.
    pub fn generate_block(
        &mut self,
        abstract_state_in: AbstractState,
        abstract_state_out: AbstractState,
    ) -> Vec<Bytecode> {
        let mut bytecode: Vec<Bytecode> = Vec::new();
        let mut state = abstract_state_in.clone();
        // Make all locals unavailable
        for (i, _) in abstract_state_in.get_locals().iter() {
            // TODO: Check that this is not a resource or reference
            let next_instruction = Bytecode::MoveLoc(*i as u8);
            state = self.apply_instruction(state.clone(), &mut bytecode, next_instruction);
        }
        // Generate block body
        loop {
            let candidates =
                self.candidate_instructions(state.clone(), abstract_state_in.get_locals().len());
            debug!("Candidates: [{:?}]", candidates);
            if candidates.is_empty() {
                warn!("No candidates found for state: [{:?}]", state);
                break;
            }
            let next_instruction = self.select_candidate(0, &state, &candidates);
            state = self.apply_instruction(state, &mut bytecode, next_instruction);
            if state.is_final() {
                break;
            }
        }
        // Add in available locals
        for (i, (token_type, availability)) in abstract_state_out.get_locals().iter() {
            if *availability == BorrowState::Available {
                let next_instruction = match token_type {
                    SignatureToken::String => Bytecode::LdStr(StringPoolIndex::new(0)),
                    SignatureToken::Address => Bytecode::LdAddr(AddressPoolIndex::new(0)),
                    SignatureToken::U64 => Bytecode::LdConst(0),
                    SignatureToken::Bool => Bytecode::LdFalse,
                    SignatureToken::ByteArray => Bytecode::LdByteArray(ByteArrayPoolIndex::new(0)),
                    _ => panic!("Unsupported return type: {:#?}", token_type),
                };
                state = self.apply_instruction(state, &mut bytecode, next_instruction);
                state = self.apply_instruction(state, &mut bytecode, Bytecode::StLoc(*i as u8));
            }
        }
        bytecode
    }

    /// Add the incoming and outgoing locals for each basic block in the control flow graph.
    /// Currently the incoming and outgoing locals are the same for each block.
    pub fn add_locals(&self, locals: &[SignatureToken], cfg: &mut CFG) {
        let locals1: HashMap<usize, (SignatureToken, BorrowState)> = locals
            .iter()
            .enumerate()
            .map(|(i, token)| (i, (token.clone(), BorrowState::Available)))
            .collect();
        for (_, block) in cfg.basic_blocks.iter_mut() {
            let locals2 = locals1.clone();
            block.locals_in = locals1.clone();
            block.locals_out = locals2;
        }
    }

    /// Construct a control flow graph that contains empty basic blocks with set incoming
    /// and outgoing locals.
    /// Currently the control flow graph is acyclic.
    pub fn build_cfg(
        &mut self,
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
            let child_1 = self.rng.gen_range(i, target_blocks);
            edges.push((i, child_1));
            // At most two children per block
            if self.rng.gen_range(0, 1) == 1 {
                let child_2 = self.rng.gen_range(i, target_blocks);
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
        self.add_locals(locals, &mut cfg);
        cfg
    }

    /// Get the serialized code offset of a basic block based on its position in the serialized
    /// instruction sequence.
    pub fn get_block_offset(&self, cfg: &CFG, block_id: BlockIDSize) -> u16 {
        let mut offset: u16 = 0;
        for i in 0..block_id {
            if let Some(block) = cfg.basic_blocks.get(&i) {
                offset += block.instructions.len() as u16;
            } else {
                panic!("Error: Invalid block_id given: {:#?}", i);
            }
        }
        offset
    }

    /// Serialize the control flow graph into a sequence of instructions. Set the offsets of branch
    /// instructions appropriately.
    pub fn serialize_cfg(&self, mut cfg: CFG) -> Vec<Bytecode> {
        let cfg_copy = cfg.clone();
        if !cfg.basic_blocks.is_empty() {
            let mut bytecode: Vec<Bytecode> = Vec::new();
            for i in 0..cfg.basic_blocks.len() {
                let block_id = i as BlockIDSize;
                let block = cfg.basic_blocks.get_mut(&block_id).unwrap();
                let last_instruction_index = block.instructions.len() - 1;
                if cfg_copy.num_children(block_id) == 2 {
                    let child_id: BlockIDSize = cfg_copy.get_children_ids(block_id)[1];
                    // The left child (fallthrough) is serialized before the right (jump)
                    let offset = self.get_block_offset(&cfg_copy, child_id);
                    match block.instructions.last() {
                        Some(Bytecode::BrTrue(_)) => {
                            block.instructions[last_instruction_index] =
                                Bytecode::BrTrue(offset as u16);
                        }
                        Some(Bytecode::BrFalse(_)) => {
                            block.instructions[last_instruction_index] =
                                Bytecode::BrFalse(offset as u16);
                        }
                        _ => panic!(
                            "Error: unsupported two target jump instruction, {:#?}",
                            block.instructions.last()
                        ),
                    };
                } else if cfg_copy.num_children(block_id) == 1 {
                    let child_id: BlockIDSize = cfg_copy.get_children_ids(block_id)[0];
                    let offset = self.get_block_offset(&cfg_copy, child_id);
                    match block.instructions.last() {
                        Some(Bytecode::Branch(_)) => {
                            block.instructions[last_instruction_index] = Bytecode::Branch(offset);
                        }
                        _ => panic!(
                            "Error: unsupported one target jump instruction, {:#?}",
                            block.instructions.last()
                        ),
                    }
                }
                bytecode.extend(block.instructions.clone());
            }
            debug!("Final bytecode: {:#?}", bytecode);
            return bytecode;
        }
        panic!("Error: CFG has no basic blocks");
    }

    /// Generate the body of a function definition given a set of starting `locals` and a target
    /// return `signature`. The sequence should contain at least `target_min` and at most
    /// `target_max` instructions.
    pub fn generate(
        &mut self,
        locals: &[SignatureToken],
        signature: &FunctionSignature,
        _target_min: usize,
        _target_max: usize,
    ) -> Vec<Bytecode> {
        let mut cfg = self.build_cfg(locals, signature, 3);
        let cfg_copy = cfg.clone();
        for (block_id, block) in cfg.basic_blocks.iter_mut() {
            let state1 = AbstractState::from_locals(block.locals_in.clone());
            let state2 = AbstractState::from_locals(block.locals_out.clone());
            let mut bytecode = self.generate_block(state1, state2.clone());
            let mut state_f = state2;
            if cfg_copy.num_children(*block_id) == 2 {
                // BrTrue, BrFalse: Add bool and branching instruction randomly
                state_f = self.apply_instruction(state_f, &mut bytecode, Bytecode::LdFalse);
                if self.rng.gen_range(0, 1) == 1 {
                    self.apply_instruction(state_f, &mut bytecode, Bytecode::BrTrue(0));
                } else {
                    self.apply_instruction(state_f, &mut bytecode, Bytecode::BrFalse(0));
                }
            } else if cfg_copy.num_children(*block_id) == 1 {
                // Branch: Add branch instruction
                self.apply_instruction(state_f, &mut bytecode, Bytecode::Branch(0));
            } else if cfg_copy.num_children(*block_id) == 0 {
                // TODO: Abort
                // Return: Add return types to last block
                for token_type in signature.return_types.iter() {
                    let next_instruction = match token_type {
                        SignatureToken::String => Bytecode::LdStr(StringPoolIndex::new(0)),
                        SignatureToken::Address => Bytecode::LdAddr(AddressPoolIndex::new(0)),
                        SignatureToken::U64 => Bytecode::LdConst(0),
                        SignatureToken::Bool => Bytecode::LdFalse,
                        SignatureToken::ByteArray => {
                            Bytecode::LdByteArray(ByteArrayPoolIndex::new(0))
                        }
                        _ => panic!("Unsupported return type: {:#?}", token_type),
                    };
                    state_f = self.apply_instruction(state_f, &mut bytecode, next_instruction);
                }
                self.apply_instruction(state_f, &mut bytecode, Bytecode::Ret);
            }
            debug!("Instructions generated: {}", bytecode.len());
            block.instructions = bytecode;
        }

        self.serialize_cfg(cfg)
    }
}
