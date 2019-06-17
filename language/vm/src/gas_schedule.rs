// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module lays out the basic abstract costing schedule for bytecode instructions.
//!
//! It is important to note that the cost schedule defined in this file does not track hashing
//! operations or other native operations; the cost of each native operation will be returned by the
//! native function itself.
use crate::file_format::Bytecode;
use std::{ops::Add, u64};

/// The underlying carrier for the gas cost
pub type GasUnits = u64;

/// Type for representing the size of our memory
pub type AbstractMemorySize = u64;

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

/// A number of instructions fall within three different tiers of gas usage. However some
/// instructions do not fall within these three categories and are separated out into their own gas
/// costs.
pub enum GasCostTable {
    Low,
    Mid,
    High,
    EmitEvent,
    GetTxnSequenceNumber,
    GetTxnPublicKey,
    GetTxnGasUnitPrice,
    GetTxnMaxGasUnits,
    GetGasRemaining,
    GetTxnSenderAddress,
    Unpack,
    Exists,
    BorrowGlobal,
    ReleaseRef,
    MoveFrom,
    MoveToSender,
    CreateAccount,
}

/// The  `GasCost` tracks:
/// - instruction cost: how much time/computational power is needed to perform the instruction
/// - memory cost: how much memory is required for the instruction, and storage overhead
/// - stack cost: how large does the value stack grow or shrink because of this operation. Note that
///   this could in a sense have ``negative'' cost; an instruction can decrease the size of the
///   stack -- however in this case we view the instruction as having zero stack cost.
#[derive(Debug)]
pub struct GasCost {
    pub instruction_gas: GasUnits,
    pub memory_gas: GasUnits,
    pub stack_gas: GasUnits,
}

/// The implementation of the `GasCostTable` provides denotations for each abstract gas tier for
/// each of the three different resources that we track.
///
/// Note that these denotations are filler (read: bogus) for now. The constants will be filled in
/// with the synthesized costs later on, or provided by a separate contract that we read in.
impl GasCostTable {
    /// Return the instruction (computational) cost for the tier.
    pub fn instr_cost(&self) -> GasUnits {
        use GasCostTable::*;
        match *self {
            Low => 1,
            Mid => 2,
            High => 3,
            EmitEvent => 4,
            GetTxnSequenceNumber => 2,
            GetTxnPublicKey => 2,
            GetTxnGasUnitPrice => 2,
            GetTxnMaxGasUnits => 2,
            GetGasRemaining => 2,
            GetTxnSenderAddress => 2,
            Unpack => 5,
            Exists => 3,
            BorrowGlobal => 3,
            ReleaseRef => 3,
            MoveFrom => 5,
            MoveToSender => 5,
            CreateAccount => 5,
        }
    }

    /// Return the memory cost for the tier.
    pub fn mem_cost(&self) -> GasUnits {
        use GasCostTable::*;
        match *self {
            Low => 0,
            Mid => 1,
            High => 2,
            EmitEvent => 3,
            GetTxnSequenceNumber => 2,
            GetTxnPublicKey => 2,
            GetTxnGasUnitPrice => 4,
            GetTxnMaxGasUnits => 4,
            GetGasRemaining => 4,
            GetTxnSenderAddress => 1,
            Unpack => 2,
            Exists => 3,
            BorrowGlobal => 4,
            ReleaseRef => 4,
            MoveFrom => 5,
            MoveToSender => 5,
            CreateAccount => 5,
        }
    }

    /// Return the stack cost for the tier.
    pub fn stack_cost(&self) -> GasUnits {
        use GasCostTable::*;
        match *self {
            Low => 0,
            Mid => 1,
            High => 2,
            EmitEvent => 0,
            GetTxnSequenceNumber => 1,
            GetTxnPublicKey => 1,
            GetTxnGasUnitPrice => 1,
            GetTxnMaxGasUnits => 1,
            GetGasRemaining => 1,
            GetTxnSenderAddress => 1,
            Unpack => 1,
            Exists => 1,
            BorrowGlobal => 1,
            ReleaseRef => 1,
            MoveFrom => 0,
            MoveToSender => 0,
            CreateAccount => 1,
        }
    }
}

// The general costing methodology that we will use for instruction cost
// determinations is as follows:
// * Each stack op (push, pop, ld, etc.) will have a gas cost of Low
// * Each primitive operation (+, -, *, jmp, etc.) will also have a gas cost of Low
// * Each function call will be charged a Mid cost
// * Each local memory operation will have a gas cost of Mid
// * Each global storage operation will have a gas cost of High
// * Examples:
//  1. gas_cost_instr(Add) = 4 * Low.value()
//     2  - one for each pop from the stack
//     1 - for the arithmetic op
//     1 - for the push of the resulting value onto the stack
//  2. gas_cost_instr(Branch(_)) = 1 * Low.value()
//     1 - A single jmp instruction, with no stack transition
//  3. gas_cost_instr(BrFalse(_)) = 2 * Low.value()
//     1 - perform one pop on the stack
//     1 - perform one jmp (possibly)
//     -> NOTE: Will we want to charge a cost for possible lack of pipelining that
//        we can perform with this instruction?
// `size_provider` provides the size for the costing, this can be e.g. the size of memory, or the
// number of arguments to the Call of Pack.
fn static_gas_cost_instr(instr: &Bytecode, size_provider: GasUnits) -> GasUnits {
    use GasCostTable::*;
    match instr {
        // pop -> pop -> op -> push | all Low
        Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Mod
        | Bytecode::Div
        | Bytecode::BitOr
        | Bytecode::BitAnd
        | Bytecode::Xor
        | Bytecode::Or
        | Bytecode::And
        | Bytecode::Not
        | Bytecode::Eq
        | Bytecode::Neq
        | Bytecode::Lt
        | Bytecode::Gt
        | Bytecode::Le
        | Bytecode::Ge => 4 * Low.instr_cost(),
        // push XOR pop XOR jmp | all Tier 0
        Bytecode::FreezeRef
        | Bytecode::LdTrue
        | Bytecode::LdFalse
        | Bytecode::LdConst(_)
        | Bytecode::BorrowLoc(_)
        | Bytecode::Branch(_)
        | Bytecode::Pop => Low.instr_cost(),
        // Read from local/pool table -> push | tier 1 -> tier 0
        Bytecode::LdStr(_) | Bytecode::LdAddr(_) | Bytecode::LdByteArray(_) => {
            Mid.instr_cost() + Low.instr_cost()
        }
        // pop -> push XOR pop -> op{assert, jmpeq} | all Low
        Bytecode::Ret | Bytecode::Assert | Bytecode::BrTrue(_) | Bytecode::BrFalse(_) => {
            2 * Low.instr_cost()
        }
        // Load from global mem -> push, High -> Low
        Bytecode::BorrowField(_) => High.instr_cost() + Low.instr_cost(),
        // Load from local mem -> push XOR pop -> write local mem | Mid -> Low XOR Low -> Mid
        Bytecode::MoveLoc(_) | Bytecode::StLoc(_) => Mid.instr_cost() + Low.instr_cost(),
        // pop -> mem op -> push XOR pop -> pop -> mem op | Low -> High
        // Since we charge for the BorrowGlobal, we don't need to charge that high of a cost for
        // ReadRef. WriteRef on the other hand will incur a possible write to global memory since
        // we can't determine if the reference is global or local at this time.
        Bytecode::ReadRef => 2 * Low.instr_cost() + Mid.instr_cost(),
        Bytecode::WriteRef => 2 * Low.instr_cost() + High.instr_cost(),
        // size_provider gives us the number of bytes that need to be copied
        // Copy bytes from locals -> push value onto stack | size_provider * Mid + Low
        Bytecode::CopyLoc(_) => Low.instr_cost() + size_provider * Mid.instr_cost(),
        // Allocate size_provider bytes for the new class -> push | size_provider * Mid + Low
        // Question: Where will we get the class info to determine the layout of
        //           the object? We will need to include that in the cost estimate for
        //           this function.
        Bytecode::Pack(_) => size_provider * Mid.instr_cost(),
        // #size_provider pops -> #size_provider writes to local memory -> fn call
        //  | size_provider * Low + size_provider * Mid + Mid
        Bytecode::Call(_) => {
            size_provider * Low.instr_cost() + (size_provider + 1) * Mid.instr_cost()
        }
        Bytecode::Unpack(_) => size_provider * Unpack.instr_cost(),
        Bytecode::CreateAccount => CreateAccount.instr_cost(),
        Bytecode::EmitEvent => EmitEvent.instr_cost(),
        Bytecode::GetTxnSenderAddress => GetTxnSenderAddress.instr_cost(),
        Bytecode::GetTxnSequenceNumber => GetTxnSequenceNumber.instr_cost(),
        Bytecode::GetTxnPublicKey => GetTxnPublicKey.instr_cost(),
        Bytecode::GetTxnGasUnitPrice => GetTxnGasUnitPrice.instr_cost(),
        Bytecode::GetTxnMaxGasUnits => GetTxnMaxGasUnits.instr_cost(),
        Bytecode::GetGasRemaining => GetGasRemaining.instr_cost(),
        Bytecode::Exists(_) => Exists.instr_cost(),
        Bytecode::BorrowGlobal(_) => BorrowGlobal.instr_cost(),
        Bytecode::ReleaseRef => ReleaseRef.instr_cost(),
        Bytecode::MoveFrom(_) => MoveFrom.instr_cost(),
        Bytecode::MoveToSender(_) => MoveToSender.instr_cost(),
    }
}

// Determine the cost to memory (in terms of size) for various operations. The
// tiers have the following meaning here:
// - Low: Don't touch any memory, just stack
// - Mid: Touch only local memory
// - High: Touch global storage
// Note that we _do not_ track the size of memory, and charge for expansion
// of this memory, nor do we track whether or not we are setting bits from zero
// or not (i.e. what Ethereum does).
fn static_gas_cost_mem(instr: &Bytecode, size_provider: GasUnits) -> GasUnits {
    use GasCostTable::*;
    match instr {
        // All of these operations don't touch memory. So have Low memory cost
        Bytecode::FreezeRef
        | Bytecode::Pop
        | Bytecode::Ret
        | Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Mod
        | Bytecode::Div
        | Bytecode::BitOr
        | Bytecode::BitAnd
        | Bytecode::Xor
        | Bytecode::Or
        | Bytecode::And
        | Bytecode::Not
        | Bytecode::Eq
        | Bytecode::Neq
        | Bytecode::Lt
        | Bytecode::Gt
        | Bytecode::Le
        | Bytecode::Ge
        | Bytecode::LdTrue
        | Bytecode::LdFalse
        | Bytecode::LdConst(_)
        | Bytecode::Assert
        | Bytecode::BrTrue(_)
        | Bytecode::BrFalse(_)
        | Bytecode::Branch(_)
        | Bytecode::CopyLoc(_) // Stored on the stack, so no overhead as such
        | Bytecode::MoveLoc(_)  // Moved, so stays the same
        | Bytecode::BorrowLoc(_) => Low.mem_cost(),
        // Call and Pack values (etc.) are allocated on the stack
         Bytecode::LdByteArray(_) | Bytecode::LdAddr(_) | Bytecode::LdStr(_) | Bytecode::Pack(_) | Bytecode::Call(_) => size_provider * Low.mem_cost(),
        // pop -> write local memory
        Bytecode::StLoc(_) => size_provider * Mid.mem_cost() + Low.mem_cost(),
        // One load from global, and push to the stack
        Bytecode::BorrowField(_) => High.mem_cost() + Low.mem_cost(),
        // We assume that all references are non-local
        Bytecode::WriteRef | Bytecode::ReadRef => High.mem_cost(),
        Bytecode::EmitEvent => size_provider * EmitEvent.mem_cost(),
        Bytecode::GetTxnSenderAddress => GetTxnSenderAddress.mem_cost(),
        Bytecode::Unpack(_) => size_provider * Unpack.mem_cost(),
        Bytecode::CreateAccount => size_provider * CreateAccount.mem_cost(),
        Bytecode::GetTxnSequenceNumber => GetTxnSequenceNumber.mem_cost(),
        Bytecode::GetTxnPublicKey => GetTxnPublicKey.mem_cost(),
        Bytecode::GetTxnGasUnitPrice => GetTxnGasUnitPrice.mem_cost(),
        Bytecode::GetTxnMaxGasUnits => GetTxnMaxGasUnits.mem_cost(),
        Bytecode::GetGasRemaining => GetGasRemaining.mem_cost(),
        Bytecode::Exists(_) => Exists.mem_cost(),
        Bytecode::BorrowGlobal(_) => size_provider * BorrowGlobal.mem_cost(),
        Bytecode::ReleaseRef => ReleaseRef.mem_cost(),
        Bytecode::MoveFrom(_) => size_provider * MoveFrom.mem_cost(),
        Bytecode::MoveToSender(_) => size_provider * MoveToSender.mem_cost(),
    }
}

// We charge a stack cost based upon how much the given bytecode will effect the stack size.
// The tiers have the following meaning here:
// - Low: Reduction in stack size of numeric constant size
// - Mid: Stack size remains constant
// - High: Push to the stack of constant size
fn static_gas_cost_stack(instr: &Bytecode, size_provider: GasUnits) -> GasUnits {
    use GasCostTable::*;
    match instr {
        Bytecode::FreezeRef
        | Bytecode::Pop
        | Bytecode::BrTrue(_)
        | Bytecode::BrFalse(_)
        | Bytecode::ReadRef
        | Bytecode::Assert => Low.stack_cost(),
        Bytecode::Ret | Bytecode::Branch(_) => Mid.stack_cost(),
        Bytecode::BorrowLoc(_)
        | Bytecode::BorrowField(_)
        | Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Mod
        | Bytecode::Div
        | Bytecode::BitOr
        | Bytecode::BitAnd
        | Bytecode::Xor
        | Bytecode::Or
        | Bytecode::And
        | Bytecode::Not
        | Bytecode::Eq
        | Bytecode::Neq
        | Bytecode::Lt
        | Bytecode::Gt
        | Bytecode::Le
        | Bytecode::Ge => 2 * Low.stack_cost() + Mid.stack_cost(),
        Bytecode::LdTrue
        | Bytecode::LdFalse
        | Bytecode::LdAddr(_)
        | Bytecode::LdByteArray(_)
        | Bytecode::LdStr(_)
        | Bytecode::LdConst(_) => High.stack_cost(),
        Bytecode::CopyLoc(_) | Bytecode::MoveLoc(_) => size_provider * High.stack_cost(),
        Bytecode::StLoc(_) => size_provider * Low.stack_cost(),
        Bytecode::Pack(_) | Bytecode::Call(_) => (size_provider + 1) * High.mem_cost(),
        Bytecode::WriteRef => 2 * Low.mem_cost(),
        Bytecode::EmitEvent => EmitEvent.stack_cost(),
        Bytecode::GetTxnSenderAddress => GetTxnSenderAddress.stack_cost(),
        Bytecode::GetTxnSequenceNumber => GetTxnSequenceNumber.stack_cost(),
        Bytecode::GetTxnPublicKey => GetTxnPublicKey.stack_cost(),
        Bytecode::GetTxnGasUnitPrice => GetTxnGasUnitPrice.stack_cost(),
        Bytecode::GetTxnMaxGasUnits => GetTxnMaxGasUnits.stack_cost(),
        Bytecode::GetGasRemaining => GetGasRemaining.stack_cost(),
        Bytecode::Unpack(_) => Unpack.stack_cost(),
        Bytecode::Exists(_) => Exists.stack_cost(),
        Bytecode::BorrowGlobal(_) => BorrowGlobal.stack_cost(),
        Bytecode::ReleaseRef => ReleaseRef.stack_cost(),
        Bytecode::MoveFrom(_) => MoveFrom.stack_cost(),
        Bytecode::MoveToSender(_) => MoveToSender.stack_cost(),
        Bytecode::CreateAccount => CreateAccount.stack_cost(),
    }
}

/// Statically cost a bytecode instruction.
///
/// Don't take into account current stack or memory size. Don't track whether references are to
/// global or local storage.
pub fn static_cost_instr(instr: &Bytecode, size_provider: GasUnits) -> GasCost {
    GasCost {
        instruction_gas: static_gas_cost_instr(instr, size_provider),
        memory_gas: static_gas_cost_mem(instr, size_provider),
        stack_gas: static_gas_cost_stack(instr, size_provider),
    }
}

/// Computes the number of words rounded up
pub fn words_in(size: AbstractMemorySize) -> AbstractMemorySize {
    // round-up div truncate
    (size + (WORD_SIZE - 1)) / WORD_SIZE
}

/// Calculate the intrinsic gas for the transaction based upon its size in bytes/words.
pub fn calculate_intrinsic_gas(transaction_size: u64) -> GasUnits {
    let min_transaction_fee = MIN_TRANSACTION_GAS_UNITS;

    if transaction_size > LARGE_TRANSACTION_CUTOFF {
        let excess = words_in(transaction_size - LARGE_TRANSACTION_CUTOFF);
        min_transaction_fee + INTRINSIC_GAS_PER_BYTE * excess
    } else {
        min_transaction_fee
    }
}

impl Add for GasCost {
    type Output = GasCost;
    fn add(self, other: GasCost) -> GasCost {
        GasCost {
            instruction_gas: self.instruction_gas + other.instruction_gas,
            memory_gas: self.memory_gas + other.memory_gas,
            stack_gas: self.stack_gas + other.stack_gas,
        }
    }
}
