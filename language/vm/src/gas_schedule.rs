// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module lays out the basic abstract costing schedule for bytecode instructions.
//!
//! It is important to note that the cost schedule defined in this file does not track hashing
//! operations or other native operations; the cost of each native operation will be returned by the
//! native function itself.
use crate::{
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, FieldDefinitionIndex, FunctionHandleIndex,
        StringPoolIndex, StructDefinitionIndex,
    },
    serializer::serialize_instruction,
};
use lazy_static::lazy_static;
use std::{collections::HashMap, u64};
use types::transaction::MAX_TRANSACTION_SIZE_IN_BYTES;

/// The underlying carrier for the gas cost
pub type GasUnits = u64;

/// Type for representing the size of our memory
pub type AbstractMemorySize = u64;

/// A newtype wrapper around the on-chain representation of an instruciton key. This is the
/// serialization of the instruction but disregarding any instruction arguments.
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct InstructionKey(pub u8);
/// The maximum size representable by AbstractMemorySize
pub const MAX_ABSTRACT_MEMORY_SIZE: AbstractMemorySize = std::u64::MAX as AbstractMemorySize;

/// The units of gas that should be charged per byte for every transaction.
pub const INTRINSIC_GAS_PER_BYTE: GasUnits = 8;

/// The minimum gas price that a transaction can be submitted with.
pub const MIN_PRICE_PER_GAS_UNIT: u64 = 0;

/// The maximum gas unit price that a transaction can be submitted with.
pub const MAX_PRICE_PER_GAS_UNIT: u64 = 10_000;

/// 1 nanosecond should equal one unit of computational gas. We bound the maximum
/// computational time of any given transaction at 10 milliseconds. We want this number and
/// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
///         MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits::MAX)
pub const MAXIMUM_NUMBER_OF_GAS_UNITS: GasUnits = 1_000_000;

/// We charge one unit of gas per-byte for the first 600 bytes
pub const MIN_TRANSACTION_GAS_UNITS: GasUnits = 600;

/// The word size that we charge by
pub const WORD_SIZE: AbstractMemorySize = 8;

/// The size in words for a non-string or address constant on the stack
pub const CONST_SIZE: AbstractMemorySize = 1;

/// The size in words for a reference on the stack
pub const REFERENCE_SIZE: AbstractMemorySize = 8;

/// The size of a struct in words
pub const STRUCT_SIZE: AbstractMemorySize = 2;

/// For V1 all accounts will be 32 words
pub const DEFAULT_ACCOUNT_SIZE: AbstractMemorySize = 32;

/// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
pub const LARGE_TRANSACTION_CUTOFF: AbstractMemorySize = 600;

/// The cost tables, keyed by the serialized form of the bytecode instruction.  We use the
/// serialized form as opposed to the instruction enum itself as the key since this will be the
/// on-chain representation of bytecode instructions in the future.
#[derive(Debug)]
pub struct CostTable {
    pub compute_table: HashMap<InstructionKey, GasUnits>,
    pub memory_table: HashMap<InstructionKey, GasUnits>,
}

impl InstructionKey {
    /// The encoding of the instruction is the serialized form of it, but disregarding the
    /// serializtion of the instructions arguments.
    pub fn new(instruction: &Bytecode) -> Self {
        let mut vec = Vec::new();
        serialize_instruction(&mut vec, instruction);
        Self(vec[0])
    }
}

impl CostTable {
    pub fn new(instrs: Vec<(Bytecode, GasUnits, GasUnits)>) -> Self {
        let mut compute_table = HashMap::new();
        let mut memory_table = HashMap::new();
        for (instr, comp_cost, mem_cost) in instrs.into_iter() {
            let code = InstructionKey::new(&instr);
            compute_table.insert(code, comp_cost);
            memory_table.insert(code, mem_cost);
        }
        Self {
            compute_table,
            memory_table,
        }
    }

    pub fn memory_gas(&self, instr: &Bytecode, size_provider: AbstractMemorySize) -> GasUnits {
        let code = InstructionKey::new(instr);
        self.memory_table.get(&code).unwrap() * size_provider
    }

    pub fn comp_gas(&self, instr: &Bytecode, size_provider: AbstractMemorySize) -> GasUnits {
        let code = InstructionKey::new(instr);
        self.memory_table.get(&code).unwrap() * size_provider
    }
}

lazy_static! {
    static ref GAS_SCHEDULE: CostTable = {
        use Bytecode::*;
        // Arguments to the instructions don't matter -- these will be removed in the
        // `encode_instruction` function.
        //
        // The second element of the tuple is the computational cost. The third element of the
        // tuple is the memory cost per-byte for the instruction.
        // TODO: At the moment the computational cost is correct, and the memory cost is not
        // correct at all (hence why they're all 1's at the moment).
        let instrs = vec![
            (MoveToSender(StructDefinitionIndex::new(0)), 774, 1),
            (GetTxnSenderAddress, 30, 1),
            (MoveFrom(StructDefinitionIndex::new(0)), 917, 1),
            (BrTrue(0), 31, 1),
            (WriteRef, 65, 1),
            (Mul, 41, 1),
            (MoveLoc(0), 41, 1),
            (And, 49, 1),
            (ReleaseRef, 28, 1),
            (GetTxnPublicKey, 41, 1),
            (Pop, 27, 1),
            (BitAnd, 44, 1),
            (ReadRef, 51, 1),
            (Sub, 44, 1),
            (BorrowField(FieldDefinitionIndex::new(0)), 58, 1),
            (Add, 45, 1),
            (CopyLoc(0), 41, 1),
            (StLoc(0), 28, 1),
            (Ret, 28, 1),
            (Lt, 49, 1),
            (LdConst(0), 29, 1),
            (Assert, 39, 1),
            (BorrowLoc(0), 45, 1),
            (LdStr(StringPoolIndex::new(0)), 52, 1),
            (LdAddr(AddressPoolIndex::new(0)), 36, 1),
            (Ge, 46, 1),
            (Xor, 46, 1),
            (Neq, 51, 1),
            (Not, 35,1),
            (Call(FunctionHandleIndex::new(0)), 197, 1),
            (Le, 47, 1),
            (CreateAccount, 1119, 1),
            (Branch(0), 10, 1),
            (Unpack(StructDefinitionIndex::new(0)), 94, 1),
            (Or, 43, 1),
            (LdFalse, 30, 1),
            (LdTrue, 29, 1),
            (GetTxnGasUnitPrice, 29, 1),
            (Mod, 42, 1),
            (BrFalse(0), 29, 1),
            (Exists(StructDefinitionIndex::new(0)), 856, 1),
            (GetGasRemaining, 32, 1),
            (BitOr, 45, 1),
            (GetTxnMaxGasUnits, 34, 1),
            (GetTxnSequenceNumber, 29, 1),
            (FreezeRef, 10, 1),
            (BorrowGlobal(StructDefinitionIndex::new(0)), 929, 1),
            (Div, 41, 1),
            (Eq, 48, 1),
            (LdByteArray(ByteArrayPoolIndex::new(0)), 56, 1),
            (Gt, 46, 1),
            (Pack(StructDefinitionIndex::new(0)), 73, 1),
            // TODO/XXX: Need to get the cost for this still
            (EmitEvent, 1, 1),
            ];
        CostTable::new(instrs)
    };
}

/// The  `GasCost` tracks:
/// - instruction cost: how much time/computational power is needed to perform the instruction
/// - memory cost: how much memory is required for the instruction, and storage overhead
#[derive(Debug)]
pub struct GasCost {
    pub instruction_gas: GasUnits,
    pub memory_gas: GasUnits,
}

/// Statically cost a bytecode instruction.
///
/// Don't take into account current stack or memory size. Don't track whether references are to
/// global or local storage.
pub fn static_cost_instr(instr: &Bytecode, size_provider: AbstractMemorySize) -> GasCost {
    GasCost {
        instruction_gas: GAS_SCHEDULE.comp_gas(instr, size_provider),
        memory_gas: GAS_SCHEDULE.memory_gas(instr, size_provider),
    }
}

/// Computes the number of words rounded up
pub fn words_in(size: AbstractMemorySize) -> AbstractMemorySize {
    precondition!(size <= MAX_ABSTRACT_MEMORY_SIZE - (WORD_SIZE + 1));
    // round-up div truncate
    (size + (WORD_SIZE - 1)) / WORD_SIZE
}

/// Calculate the intrinsic gas for the transaction based upon its size in bytes/words.
pub fn calculate_intrinsic_gas(transaction_size: u64) -> GasUnits {
    precondition!(transaction_size <= MAX_TRANSACTION_SIZE_IN_BYTES as u64);
    let min_transaction_fee = MIN_TRANSACTION_GAS_UNITS;

    if transaction_size > LARGE_TRANSACTION_CUTOFF {
        let excess = words_in(transaction_size - LARGE_TRANSACTION_CUTOFF);
        min_transaction_fee + INTRINSIC_GAS_PER_BYTE * excess
    } else {
        min_transaction_fee
    }
}
