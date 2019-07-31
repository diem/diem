// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{abstract_state::AbstractState, summaries};
use rand::{rngs::StdRng, Rng, SeedableRng};
use vm::file_format::{
    AddressPoolIndex, Bytecode, FunctionSignature, SignatureToken, StringPoolIndex,
};

/// This type represents bytecode instructions that take a `u8`
type U8ToBytecode = fn(u8) -> Bytecode;

/// This type represents bytecode instructions that take a `u64`
type U64ToBytecode = fn(u64) -> Bytecode;

/// This type represents bytecode instructions that take a `StringPoolIndex`
type StringPoolIndexToBytecode = fn(StringPoolIndex) -> Bytecode;

/// This type represents bytecode instructions that take a `AddressPoolIndex`
type AddressPoolIndexToBytecode = fn(AddressPoolIndex) -> Bytecode;

/// There are three types of bytecode instructions
#[derive(Debug, Clone)]
enum BytecodeType {
    /// Instructions that do not take an argument
    NoArg(Bytecode),

    /// Instructions that take a `u8`
    U8(U8ToBytecode),

    /// Instructions that take a `u64`
    U64(U64ToBytecode),

    /// Instructions that take a `StringPoolIndex`
    StringPoolIndex(StringPoolIndexToBytecode),

    /// Instructions that take an `AddressPoolIndex`
    AddressPoolIndex(AddressPoolIndexToBytecode),
}

/// Generates a sequence of bytecode instructions.
/// This generator has:
/// - `instructions`: A list of bytecode instructions to use for generation
/// - `rng`: A random number generator for uniform random choice of next instruction
#[derive(Debug, Clone)]
pub struct BytecodeGenerator {
    instructions: Vec<BytecodeType>,
    rng: StdRng,
}

impl BytecodeGenerator {
    /// The `BytecodeGenerator` is instantiated with a seed to use with
    /// its random number generator.
    pub fn new(seed: [u8; 32]) -> Self {
        let instructions: Vec<BytecodeType> = vec![
            BytecodeType::NoArg(Bytecode::Pop),
            BytecodeType::U64(Bytecode::LdConst),
            BytecodeType::StringPoolIndex(Bytecode::LdStr),
            BytecodeType::AddressPoolIndex(Bytecode::LdAddr),
            BytecodeType::NoArg(Bytecode::LdTrue),
            BytecodeType::NoArg(Bytecode::LdFalse),
            BytecodeType::U8(Bytecode::CopyLoc),
            BytecodeType::U8(Bytecode::MoveLoc),
            BytecodeType::U8(Bytecode::StLoc),
            BytecodeType::NoArg(Bytecode::Add),
            BytecodeType::NoArg(Bytecode::Sub),
            BytecodeType::NoArg(Bytecode::Mul),
            BytecodeType::NoArg(Bytecode::Div),
            BytecodeType::NoArg(Bytecode::Mod),
            BytecodeType::NoArg(Bytecode::BitAnd),
            BytecodeType::NoArg(Bytecode::BitOr),
            BytecodeType::NoArg(Bytecode::Xor),
            BytecodeType::NoArg(Bytecode::Or),
            BytecodeType::NoArg(Bytecode::And),
            BytecodeType::NoArg(Bytecode::Not),
            BytecodeType::NoArg(Bytecode::Eq),
            BytecodeType::NoArg(Bytecode::Neq),
            BytecodeType::NoArg(Bytecode::Lt),
            BytecodeType::NoArg(Bytecode::Gt),
            BytecodeType::NoArg(Bytecode::Le),
            BytecodeType::NoArg(Bytecode::Ge),
            BytecodeType::NoArg(Bytecode::GetTxnGasUnitPrice),
            BytecodeType::NoArg(Bytecode::GetTxnMaxGasUnits),
            BytecodeType::NoArg(Bytecode::GetGasRemaining),
            BytecodeType::NoArg(Bytecode::GetTxnSequenceNumber),
        ];
        let generator = StdRng::from_seed(seed);
        Self {
            instructions,
            rng: generator,
        }
    }

    /// Given an `AbstractState`, `state`, and a the number of locals the function has,
    /// this function returns a list of instructions whose preconditions are satisfied for
    /// the state.
    fn candidate_instructions(&mut self, state: AbstractState, locals_len: usize) -> Vec<Bytecode> {
        let mut matches: Vec<Bytecode> = Vec::new();
        let instructions = &self.instructions;
        for instruction in instructions.iter() {
            let instruction: Bytecode = match instruction {
                BytecodeType::NoArg(instruction) => instruction.clone(),
                BytecodeType::U8(instruction) => {
                    // Generate a random index into the locals
                    let local_index: u8 = self.rng.gen_range(0, locals_len as u8);
                    instruction(local_index)
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
            };
            let summary = summaries::instruction_summary(instruction.clone());
            let unsatisfied_preconditions = summary
                .preconditions
                .iter()
                .any(|precondition| !precondition(&state));
            if !unsatisfied_preconditions {
                matches.push(instruction);
            }
        }
        matches
    }

    /// Transition an abstract state, `state` to the next state by applying all of the effects
    /// of a particular bytecode instruction, `instruction`.
    fn abstract_step(&self, state: AbstractState, instruction: Bytecode) -> AbstractState {
        summaries::instruction_summary(instruction.clone())
            .effects
            .iter()
            .fold(state.clone(), |acc, effect| effect(&acc))
    }

    /// Return a sequence of bytecode instructions given a set of `locals` and a target return
    /// `_signature`. The sequence should contain at least `target_min` and at most `target_max`
    /// instructions.
    pub fn generate(
        &mut self,
        locals: &[SignatureToken],
        signature: &FunctionSignature,
        target_min: usize,
        target_max: usize,
    ) -> Vec<Bytecode> {
        let mut bytecode: Vec<Bytecode> = Vec::new();
        let mut state: AbstractState = AbstractState::new(locals);
        loop {
            debug!("Bytecode: [{:?}]", bytecode);
            debug!("AbstractState: [{:?}]", state);
            let candidates = self.candidate_instructions(state.clone(), locals.len());
            debug!("Candidates: [{:?}]", candidates);
            if candidates.is_empty() {
                warn!("No candidates found for state: [{:?}]", state);
                break;
            }
            let next_instruction_index = self.rng.gen_range(0, candidates.len());
            let next_instruction = candidates[next_instruction_index].clone();
            debug!("Next instr: {:?}", next_instruction);
            state = self.abstract_step(state, next_instruction.clone());
            debug!("New state: {:?}", state);
            bytecode.push(next_instruction);
            debug!("**********************");
            if bytecode.len() >= target_min && state.is_final() || bytecode.len() >= target_max {
                info!("Instructions generated: {}", bytecode.len());
                break;
            }
        }
        for return_type in signature.return_types.iter() {
            println!("{:?}", return_type);
            match return_type {
                SignatureToken::String => bytecode.push(Bytecode::LdStr(StringPoolIndex::new(0))),
                SignatureToken::Address => {
                    bytecode.push(Bytecode::LdAddr(AddressPoolIndex::new(0)))
                }
                _ => panic!("Unsupported return type!"),
            }
        }
        bytecode.push(Bytecode::Ret);
        bytecode
    }
}
