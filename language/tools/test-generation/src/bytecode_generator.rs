// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    abstract_state::{AbstractState, BorrowState},
    config::{MAX_CFG_BLOCKS, MUTATION_TOLERANCE, NEGATE_PRECONDITIONS, NEGATION_PROBABILITY},
    control_flow_graph::CFG,
    summaries,
};
use rand::{rngs::StdRng, FromEntropy, Rng, SeedableRng};
use utils::inhabitation::inhabit_with_bytecode_seq;
use vm::file_format::{
    AddressPoolIndex, ByteArrayPoolIndex, Bytecode, CodeOffset, CompiledModuleMut,
    FieldDefinitionIndex, FunctionHandleIndex, FunctionSignature, LocalIndex, LocalsSignatureIndex,
    SignatureToken, StructDefinitionIndex, TableIndex, UserStringIndex,
};

/// This type represents bytecode instructions that take a `LocalIndex`
type LocalIndexToBytecode = fn(LocalIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `u16`
type CodeOffsetToBytecode = fn(CodeOffset) -> Bytecode;

/// This type represents bytecode instructions that take a `u64`
type U64ToBytecode = fn(u64) -> Bytecode;

/// This type represents bytecode instructions that take a `UserStringIndex`
type UserStringIndexToBytecode = fn(UserStringIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `AddressPoolIndex`
type AddressPoolIndexToBytecode = fn(AddressPoolIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `ByteArrayPoolIndex`
type ByteArrayPoolIndexToBytecode = fn(ByteArrayPoolIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `StructDefinitionIndex`
/// and a `LocalsSignatureIndex`
type StructAndLocalIndexToBytecode = fn(StructDefinitionIndex, LocalsSignatureIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `FieldDefinitionIndex``
type FieldDefinitionIndexToBytecode = fn(FieldDefinitionIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `FunctionHandleIndex`
/// and a `LocalsSignatureIndex`
type FunctionAndLocalIndexToBytecode = fn(FunctionHandleIndex, LocalsSignatureIndex) -> Bytecode;

/// There are six types of bytecode instructions
#[derive(Debug, Clone)]
enum BytecodeType {
    /// Instructions that do not take an argument
    NoArg(Bytecode),

    /// Instructions that take a `LocalIndex`
    LocalIndex(LocalIndexToBytecode),

    /// Instructions that take a `CodeOffset`
    CodeOffset(CodeOffsetToBytecode),

    /// Instructions that take a `u64`
    U64(U64ToBytecode),

    /// Instructions that take a `UserStringIndex`
    UserStringIndex(UserStringIndexToBytecode),

    /// Instructions that take an `AddressPoolIndex`
    AddressPoolIndex(AddressPoolIndexToBytecode),

    /// Instructions that take a `ByteArrayPoolIndex`
    ByteArrayPoolIndex(ByteArrayPoolIndexToBytecode),

    /// Instructions that take a `StructDefinitionIndex` and a `LocalsSignatureIndex`
    StructAndLocalIndex(StructAndLocalIndexToBytecode),

    /// Instructions that take a `FieldDefinitionIndex`
    FieldDefinitionIndex(FieldDefinitionIndexToBytecode),

    /// Instructions that take a `FunctionHandleIndex` and a `LocalsSignatureIndex`
    FunctionAndLocalIndex(FunctionAndLocalIndexToBytecode),
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
                BytecodeType::UserStringIndex(Bytecode::LdStr),
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
            (
                StackEffect::Add,
                BytecodeType::LocalIndex(Bytecode::CopyLoc),
            ),
            (
                StackEffect::Add,
                BytecodeType::LocalIndex(Bytecode::MoveLoc),
            ),
            (StackEffect::Sub, BytecodeType::LocalIndex(Bytecode::StLoc)),
            (
                StackEffect::Add,
                BytecodeType::LocalIndex(Bytecode::MutBorrowLoc),
            ),
            (
                StackEffect::Add,
                BytecodeType::LocalIndex(Bytecode::ImmBorrowLoc),
            ),
            (StackEffect::Nop, BytecodeType::NoArg(Bytecode::ReadRef)),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::WriteRef)),
            (StackEffect::Nop, BytecodeType::NoArg(Bytecode::FreezeRef)),
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
            (
                StackEffect::Add,
                BytecodeType::NoArg(Bytecode::GetTxnSenderAddress),
            ),
            (
                StackEffect::Add,
                BytecodeType::NoArg(Bytecode::GetTxnPublicKey),
            ),
            (
                StackEffect::Nop,
                BytecodeType::StructAndLocalIndex(Bytecode::Pack),
            ),
            (
                StackEffect::Nop,
                BytecodeType::StructAndLocalIndex(Bytecode::Unpack),
            ),
            (
                StackEffect::Nop,
                BytecodeType::StructAndLocalIndex(Bytecode::Exists),
            ),
            (
                StackEffect::Add,
                BytecodeType::StructAndLocalIndex(Bytecode::MoveFrom),
            ),
            (
                StackEffect::Sub,
                BytecodeType::StructAndLocalIndex(Bytecode::MoveToSender),
            ),
            (
                StackEffect::Nop,
                BytecodeType::StructAndLocalIndex(Bytecode::MutBorrowGlobal),
            ),
            (
                StackEffect::Nop,
                BytecodeType::StructAndLocalIndex(Bytecode::ImmBorrowGlobal),
            ),
            (
                StackEffect::Nop,
                BytecodeType::FieldDefinitionIndex(Bytecode::MutBorrowField),
            ),
            (
                StackEffect::Nop,
                BytecodeType::FieldDefinitionIndex(Bytecode::ImmBorrowField),
            ),
            (
                StackEffect::Nop,
                BytecodeType::FunctionAndLocalIndex(Bytecode::Call),
            ),
            (StackEffect::Nop, BytecodeType::CodeOffset(Bytecode::Branch)),
            (StackEffect::Sub, BytecodeType::CodeOffset(Bytecode::BrTrue)),
            (
                StackEffect::Sub,
                BytecodeType::CodeOffset(Bytecode::BrFalse),
            ),
            (StackEffect::Sub, BytecodeType::NoArg(Bytecode::Abort)),
            (StackEffect::Nop, BytecodeType::NoArg(Bytecode::Ret)),
        ];
        let generator = seed.map_or_else(StdRng::from_entropy, StdRng::from_seed);
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
        module: CompiledModuleMut,
    ) -> Vec<(StackEffect, Bytecode)> {
        let mut matches: Vec<(StackEffect, Bytecode)> = Vec::new();
        let instructions = &self.instructions;
        // TODO: Remove this when we can generate type parameters
        let empty_ty_param_index = module
            .locals_signatures
            .iter()
            .position(|sig| sig.is_empty())
            .expect("locals signatures must have an empty locals signature");
        for (stack_effect, instruction) in instructions.iter() {
            let instruction: Bytecode = match instruction {
                BytecodeType::NoArg(instruction) => instruction.clone(),
                BytecodeType::LocalIndex(instruction) => {
                    // Generate a random index into the locals
                    if locals_len > 0 {
                        instruction(self.rng.gen_range(0, locals_len) as LocalIndex)
                    } else {
                        instruction(0)
                    }
                }
                BytecodeType::CodeOffset(instruction) => {
                    // Set 0 as the offset. This will be set correctly during serialization
                    instruction(0)
                }
                BytecodeType::U64(instruction) => {
                    // Generate a random u64 constant to load
                    instruction(self.rng.gen_range(0, u64::max_value()))
                }
                BytecodeType::UserStringIndex(instruction) => {
                    // Select a random user string
                    instruction(UserStringIndex::new(
                        self.rng.gen_range(0, module.user_strings.len()) as TableIndex,
                    ))
                }
                BytecodeType::AddressPoolIndex(instruction) => {
                    // Select a random address from the module's address pool
                    instruction(AddressPoolIndex::new(
                        self.rng.gen_range(0, module.address_pool.len()) as TableIndex,
                    ))
                }
                BytecodeType::ByteArrayPoolIndex(instruction) => {
                    // Select a random byte array from the module's byte array pool
                    instruction(ByteArrayPoolIndex::new(
                        self.rng.gen_range(0, module.byte_array_pool.len()) as TableIndex,
                    ))
                }
                BytecodeType::StructAndLocalIndex(instruction) => {
                    // Select a random struct definition and local signature
                    instruction(
                        StructDefinitionIndex::new(
                            self.rng.gen_range(0, module.struct_defs.len()) as TableIndex
                        ),
                        // TODO: Need to generate a proper generic call eventually
                        LocalsSignatureIndex::new(empty_ty_param_index as TableIndex),
                        //LocalsSignatureIndex::new(
                        //self.rng.gen_range(0, module.locals_signatures.len()) as TableIndex,
                        //),
                    )
                }
                BytecodeType::FieldDefinitionIndex(instruction) => {
                    // Select a field definition from the module's field definitions
                    instruction(FieldDefinitionIndex::new(
                        self.rng.gen_range(0, module.field_defs.len()) as TableIndex,
                    ))
                }
                BytecodeType::FunctionAndLocalIndex(instruction) => {
                    // Select a random function handle and local signature
                    instruction(
                        FunctionHandleIndex::new(
                            self.rng.gen_range(0, module.function_handles.len()) as TableIndex,
                        ),
                        // TODO: Need to generate a proper generic call eventually
                        LocalsSignatureIndex::new(empty_ty_param_index as TableIndex),
                        //LocalsSignatureIndex::new(
                        //self.rng.gen_range(0, module.locals_signatures.len()) as TableIndex,
                        //),
                    )
                }
            };
            let summary = summaries::instruction_summary(instruction.clone());
            let unsatisfied_preconditions = summary
                .preconditions
                .iter()
                .filter(|precondition| !precondition(&state))
                .count();
            if (NEGATE_PRECONDITIONS
                && !summary.preconditions.is_empty()
                && unsatisfied_preconditions > self.rng.gen_range(0, summary.preconditions.len())
                && self.rng.gen_range(0, 101) > 100 - (NEGATION_PROBABILITY * 100.0) as u8)
                || unsatisfied_preconditions == 0
            {
                // The size of matches cannot be greater than the number of bytecode instructions
                verify!(matches.len() < usize::max_value());
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
    ) -> Result<Bytecode, String> {
        debug!("Candidates: {:?}", candidates);
        let stack_len = state.stack_len();
        let prob_add = if stack_len > return_len {
            MUTATION_TOLERANCE / (stack_len as f32)
        } else {
            1.0
        };
        debug!("Pr[add] = {:?}", prob_add);
        let next_instruction_index;
        if self.rng.gen_range(0.0, 1.0) <= prob_add {
            let add_candidates: Vec<Bytecode> = candidates
                .iter()
                .filter(|(stack_effect, _)| {
                    *stack_effect == StackEffect::Add || *stack_effect == StackEffect::Nop
                })
                .map(|(_, candidate)| candidate)
                .cloned()
                .collect();
            // Add candidates should not be empty unless the list of bytecode instructions is
            // changed
            if add_candidates.is_empty() {
                return Err("Could not find valid add candidate".to_string());
            }
            next_instruction_index = self.rng.gen_range(0, add_candidates.len());
            Ok(add_candidates[next_instruction_index].clone())
        } else {
            let sub_candidates: Vec<Bytecode> = candidates
                .iter()
                .filter(|(stack_effect, _)| {
                    *stack_effect == StackEffect::Sub || *stack_effect == StackEffect::Nop
                })
                .map(|(_, candidate)| candidate)
                .cloned()
                .collect();
            // Sub candidates should not be empty unless the list of bytecode instructions is
            // changed
            if sub_candidates.is_empty() {
                return Err("Could not find sub valid candidate".to_string());
            }
            next_instruction_index = self.rng.gen_range(0, sub_candidates.len());
            Ok(sub_candidates[next_instruction_index].clone())
        }
    }

    /// Transition an abstract state, `state` to the next state by applying all of the effects
    /// of a particular bytecode instruction, `instruction`.
    fn abstract_step(&self, state: AbstractState, instruction: Bytecode) -> AbstractState {
        let should_error = summaries::instruction_summary(instruction.clone())
            .preconditions
            .iter()
            .any(|precondition| !precondition(&state));
        if should_error {
            debug!("Reached abort state");
            let mut state = state.clone();
            state.abort();
        }
        summaries::instruction_summary(instruction)
            .effects
            .iter()
            .fold(state, |acc, effect| {
                effect(&acc).unwrap_or_else(|err| {
                    if NEGATE_PRECONDITIONS {
                        // Ignore the effect
                        acc
                    } else {
                        unreachable!("Error applying instruction effect: {}", err);
                    }
                })
            })
    }

    /// Transition an abstract state, `state` to the next state and add the instruction
    /// to the bytecode sequence
    pub fn apply_instruction(
        &self,
        mut state: AbstractState,
        bytecode: &mut Vec<Bytecode>,
        instruction: Bytecode,
    ) -> AbstractState {
        // Bytecode will never be generated this large
        assume!(bytecode.len() < usize::max_value());
        debug!("**********************");
        debug!("State1: {}", state);
        debug!("Next instr: {:?}", instruction);
        state = self.abstract_step(state, instruction.clone());
        debug!("State2: {}", state);
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
        module: CompiledModuleMut,
    ) -> Vec<Bytecode> {
        debug!("Abstract state in: {}", abstract_state_in.clone());
        debug!("Abstract state out: {}", abstract_state_out.clone());
        let mut bytecode: Vec<Bytecode> = Vec::new();
        let mut state = abstract_state_in.clone();
        // Generate block body
        loop {
            let candidates = self.candidate_instructions(
                state.clone(),
                abstract_state_in.get_locals().len(),
                module.clone(),
            );
            if candidates.is_empty() {
                warn!("No candidates found for state: [{:?}]", state);
                break;
            }
            match self.select_candidate(0, &state, &candidates) {
                Ok(next_instruction) => {
                    state = self.apply_instruction(state, &mut bytecode, next_instruction);
                    if state.is_final() {
                        break;
                    } else if state.has_aborted() {
                        state = self.apply_instruction(state, &mut bytecode, Bytecode::LdConst(0));
                        self.apply_instruction(state, &mut bytecode, Bytecode::Abort);
                        return bytecode;
                    }
                }
                Err(err) => {
                    // Could not complete the bytecode sequence; reset to empty
                    error!("{}", err);
                    return Vec::new();
                }
            }
        }
        // Fix local availability
        for (i, (abstract_value, target_availability)) in abstract_state_out.get_locals().iter() {
            if let Some((_, current_availability)) = state.local_get(*i) {
                if *target_availability == BorrowState::Available
                    && *current_availability == BorrowState::Unavailable
                {
                    let next_instructions =
                        inhabit_with_bytecode_seq(&module, &abstract_value.token);
                    state = next_instructions
                        .into_iter()
                        .fold(state, |state, instruction| {
                            self.apply_instruction(state, &mut bytecode, instruction)
                        });
                    state = self.apply_instruction(state, &mut bytecode, Bytecode::StLoc(*i as u8));
                } else if *target_availability == BorrowState::Unavailable
                    && *current_availability == BorrowState::Available
                {
                    state =
                        self.apply_instruction(state, &mut bytecode, Bytecode::MoveLoc(*i as u8));
                    state = self.apply_instruction(state, &mut bytecode, Bytecode::Pop);
                }
            } else {
                unreachable!("Target locals out contains new local");
            }
        }
        bytecode
    }

    pub fn generate_module(&mut self, module: &mut CompiledModuleMut) {
        let frozen_module = module.clone();
        for fdef in module.function_defs.iter_mut() {
            let f_handle = &module.function_handles[fdef.function.0 as usize].clone();
            let func_sig = module.function_signatures[f_handle.signature.0 as usize].clone();
            let locals_sigs = module.locals_signatures[fdef.code.locals.0 as usize]
                .0
                .clone();
            fdef.code.code = self.generate(
                &locals_sigs,
                &func_sig,
                &fdef.acquires_global_resources,
                frozen_module.clone(),
            );
        }
    }

    /// Generate the body of a function definition given a set of starting `locals` and a target
    /// return `signature`. The sequence should contain at least `target_min` and at most
    /// `target_max` instructions.
    pub fn generate(
        &mut self,
        locals: &[SignatureToken],
        signature: &FunctionSignature,
        acquires_global_resources: &[StructDefinitionIndex],
        module: CompiledModuleMut,
    ) -> Vec<Bytecode> {
        let number_of_blocks = self.rng.gen_range(1, MAX_CFG_BLOCKS + 1);
        // The number of basic blocks must be at least one based on the
        // generation range.
        assume!(number_of_blocks > 0);
        let mut cfg = CFG::new(&mut self.rng, locals, signature, number_of_blocks);
        let cfg_copy = cfg.clone();
        for (block_id, block) in cfg.get_basic_blocks_mut().iter_mut() {
            debug!(
                "+++++++++++++++++ Starting new block: {} +++++++++++++++++",
                block_id
            );
            let state1 = AbstractState::from_locals(
                module.clone(),
                block.get_locals_in().clone(),
                acquires_global_resources.to_vec(),
            );
            let state2 = AbstractState::from_locals(
                module.clone(),
                block.get_locals_out().clone(),
                acquires_global_resources.to_vec(),
            );
            let mut bytecode = self.generate_block(state1, state2.clone(), module.clone());
            let mut state_f = state2;
            if state_f.has_aborted() {
                block.set_instructions(bytecode);
                continue;
            }
            if cfg_copy.num_children(*block_id) == 2 {
                // BrTrue, BrFalse: Add bool and branching instruction randomly
                state_f = self.apply_instruction(state_f, &mut bytecode, Bytecode::LdFalse);
                if self.rng.gen_bool(0.5) {
                    self.apply_instruction(state_f, &mut bytecode, Bytecode::BrTrue(0));
                } else {
                    self.apply_instruction(state_f, &mut bytecode, Bytecode::BrFalse(0));
                }
            } else if cfg_copy.num_children(*block_id) == 1 {
                // Branch: Add branch instruction
                self.apply_instruction(state_f, &mut bytecode, Bytecode::Branch(0));
            } else if cfg_copy.num_children(*block_id) == 0 {
                // Return: Add return types to last block
                for token_type in signature.return_types.iter() {
                    let next_instructions = inhabit_with_bytecode_seq(&module, &token_type);
                    debug!(
                        "Return value instructions: {:#?} for token {:#?}",
                        next_instructions, &token_type
                    );
                    state_f =
                        next_instructions
                            .into_iter()
                            .fold(state_f, |state_f, instruction| {
                                self.apply_instruction(state_f, &mut bytecode, instruction)
                            });
                }
                self.apply_instruction(state_f, &mut bytecode, Bytecode::Ret);
            }
            block.set_instructions(bytecode);
        }
        // The CFG will be non-empty if we set the number of basic blocks to generate
        // to be non-zero
        verify!(number_of_blocks > 0 || cfg.get_basic_blocks().is_empty());
        cfg.serialize()
    }
}
