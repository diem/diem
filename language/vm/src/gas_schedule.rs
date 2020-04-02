// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module lays out the basic abstract costing schedule for bytecode instructions.
//!
//! It is important to note that the cost schedule defined in this file does not track hashing
//! operations or other native operations; the cost of each native operation will be returned by the
//! native function itself.
use crate::file_format::{
    AddressPoolIndex, ByteArrayPoolIndex, Bytecode, FieldHandleIndex, FieldInstantiationIndex,
    FunctionHandleIndex, FunctionInstantiationIndex, StructDefInstantiationIndex,
    StructDefinitionIndex, NUMBER_OF_NATIVE_FUNCTIONS,
};
pub use crate::file_format_common::Opcodes;
use libra_types::transaction::MAX_TRANSACTION_SIZE_IN_BYTES;
use move_core_types::identifier::Identifier;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Div, Mul, Sub},
    u64,
};

/// The underlying carrier for gas-related units and costs. Data with this type should not be
/// manipulated directly, but instead be manipulated using the newtype wrappers defined around
/// them and the functions defined in the `GasAlgebra` trait.
pub type GasCarrier = u64;

/// A trait encoding the operations permitted on the underlying carrier for the gas unit, and how
/// other gas-related units can interact with other units -- operations can only be performed
/// across units with the same underlying carrier (i.e. as long as the underlying data is
/// the same).
pub trait GasAlgebra<GasCarrier>: Sized
where
    GasCarrier: Add<Output = GasCarrier>
        + Sub<Output = GasCarrier>
        + Div<Output = GasCarrier>
        + Mul<Output = GasCarrier>
        + Copy,
{
    /// Project a value into the gas algebra.
    fn new(carrier: GasCarrier) -> Self;

    /// Get the carrier.
    fn get(&self) -> GasCarrier;

    /// Map a function `f` of one argument over the underlying data.
    fn map<F: Fn(GasCarrier) -> GasCarrier>(self, f: F) -> Self {
        Self::new(f(self.get()))
    }

    /// Map a function `f` of two arguments over the underlying carrier. Note that this function
    /// can take two different implementations of the trait -- one for `self` the other for the
    /// second argument. But, we enforce that they have the same underlying carrier.
    fn map2<F: Fn(GasCarrier, GasCarrier) -> GasCarrier>(
        self,
        other: impl GasAlgebra<GasCarrier>,
        f: F,
    ) -> Self {
        Self::new(f(self.get(), other.get()))
    }

    /// Apply a function `f` of two arguments to the carrier. Since `f` is not an endomophism, we
    /// return the resulting value, as opposed to the result wrapped up in ourselves.
    fn app<T, F: Fn(GasCarrier, GasCarrier) -> T>(
        &self,
        other: &impl GasAlgebra<GasCarrier>,
        f: F,
    ) -> T {
        f(self.get(), other.get())
    }

    /// We allow casting between GasAlgebras as long as they have the same underlying carrier --
    /// i.e. they use the same type to store the underlying value.
    fn unitary_cast<T: GasAlgebra<GasCarrier>>(self) -> T {
        T::new(self.get())
    }

    /// Add the two `GasAlgebra`s together.
    fn add(self, right: impl GasAlgebra<GasCarrier>) -> Self {
        self.map2(right, Add::add)
    }

    /// Subtract one `GasAlgebra` from the other.
    fn sub(self, right: impl GasAlgebra<GasCarrier>) -> Self {
        self.map2(right, Sub::sub)
    }

    /// Multiply two `GasAlgebra`s together.
    fn mul(self, right: impl GasAlgebra<GasCarrier>) -> Self {
        self.map2(right, Mul::mul)
    }

    /// Divide one `GasAlgebra` by the other.
    fn div(self, right: impl GasAlgebra<GasCarrier>) -> Self {
        self.map2(right, Div::div)
    }
}

// We would really like to be able to implement the standard arithmetic traits over the GasAlgebra
// trait, but that isn't possible.
macro_rules! define_gas_unit {
    {
        name: $name: ident,
        carrier: $carrier: ty,
        doc: $comment: literal
    } => {
        #[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
        #[doc=$comment]
        pub struct $name<GasCarrier>(GasCarrier);
        impl GasAlgebra<$carrier> for $name<$carrier> {
            fn new(c: GasCarrier) -> Self {
                Self(c)
            }
            fn get(&self) -> GasCarrier {
                self.0
            }
        }
    }
}

define_gas_unit! {
    name: AbstractMemorySize,
    carrier: GasCarrier,
    doc: "A newtype wrapper that represents the (abstract) memory size that the instruction will take up."
}

define_gas_unit! {
    name: GasUnits,
    carrier: GasCarrier,
    doc: "A newtype wrapper around the underlying carrier for the gas cost."
}

define_gas_unit! {
    name: GasPrice,
    carrier: GasCarrier,
    doc: "A newtype wrapper around the gas price for each unit of gas consumed."
}

/// The cost per-byte written to global storage.
/// TODO: Fill this in with a proper number once it's determined.
pub const GLOBAL_MEMORY_PER_BYTE_COST: GasUnits<GasCarrier> = GasUnits(8);

/// The cost per-byte written to storage.
/// TODO: Fill this in with a proper number once it's determined.
pub const GLOBAL_MEMORY_PER_BYTE_WRITE_COST: GasUnits<GasCarrier> = GasUnits(8);

/// The maximum size representable by AbstractMemorySize
pub const MAX_ABSTRACT_MEMORY_SIZE: AbstractMemorySize<GasCarrier> =
    AbstractMemorySize(std::u64::MAX);

/// The units of gas that should be charged per byte for every transaction.
pub const INTRINSIC_GAS_PER_BYTE: GasUnits<GasCarrier> = GasUnits(8);

/// The minimum gas price that a transaction can be submitted with.
pub const MIN_PRICE_PER_GAS_UNIT: GasPrice<GasCarrier> = GasPrice(0);

/// The maximum gas unit price that a transaction can be submitted with.
pub const MAX_PRICE_PER_GAS_UNIT: GasPrice<GasCarrier> = GasPrice(10_000);

/// 1 nanosecond should equal one unit of computational gas. We bound the maximum
/// computational time of any given transaction at 10 milliseconds. We want this number and
/// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
///         MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
pub const MAXIMUM_NUMBER_OF_GAS_UNITS: GasUnits<GasCarrier> = GasUnits(1_000_000);

/// We charge one unit of gas per-byte for the first 600 bytes
pub const MIN_TRANSACTION_GAS_UNITS: GasUnits<GasCarrier> = GasUnits(600);

/// The word size that we charge by
pub const WORD_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(8);

/// The size in words for a non-string or address constant on the stack
pub const CONST_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(1);

/// The size in words for a reference on the stack
pub const REFERENCE_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(8);

/// The size of a struct in words
pub const STRUCT_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(2);

/// For V1 all accounts will be 32 words
pub const DEFAULT_ACCOUNT_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(32);

/// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
pub const LARGE_TRANSACTION_CUTOFF: AbstractMemorySize<GasCarrier> = AbstractMemorySize(600);

pub static GAS_SCHEDULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

/// The encoding of the instruction is the serialized form of it, but disregarding the
/// serialization of the instruction's argument(s).
pub fn instruction_key(instruction: &Bytecode) -> u8 {
    use Bytecode::*;
    let opcode = match instruction {
        Pop => Opcodes::POP,
        Ret => Opcodes::RET,
        BrTrue(_) => Opcodes::BR_TRUE,
        BrFalse(_) => Opcodes::BR_FALSE,
        Branch(_) => Opcodes::BRANCH,
        LdU8(_) => Opcodes::LD_U8,
        LdU64(_) => Opcodes::LD_U64,
        LdU128(_) => Opcodes::LD_U128,
        CastU8 => Opcodes::CAST_U8,
        CastU64 => Opcodes::CAST_U64,
        CastU128 => Opcodes::CAST_U128,
        LdByteArray(_) => Opcodes::LD_BYTEARRAY,
        LdAddr(_) => Opcodes::LD_ADDR,
        LdTrue => Opcodes::LD_TRUE,
        LdFalse => Opcodes::LD_FALSE,
        CopyLoc(_) => Opcodes::COPY_LOC,
        MoveLoc(_) => Opcodes::MOVE_LOC,
        StLoc(_) => Opcodes::ST_LOC,
        Call(_) => Opcodes::CALL,
        CallGeneric(_) => Opcodes::CALL_GENERIC,
        Pack(_) => Opcodes::PACK,
        PackGeneric(_) => Opcodes::PACK_GENERIC,
        Unpack(_) => Opcodes::UNPACK,
        UnpackGeneric(_) => Opcodes::UNPACK_GENERIC,
        ReadRef => Opcodes::READ_REF,
        WriteRef => Opcodes::WRITE_REF,
        FreezeRef => Opcodes::FREEZE_REF,
        MutBorrowLoc(_) => Opcodes::MUT_BORROW_LOC,
        ImmBorrowLoc(_) => Opcodes::IMM_BORROW_LOC,
        MutBorrowField(_) => Opcodes::MUT_BORROW_FIELD,
        MutBorrowFieldGeneric(_) => Opcodes::MUT_BORROW_FIELD_GENERIC,
        ImmBorrowField(_) => Opcodes::IMM_BORROW_FIELD,
        ImmBorrowFieldGeneric(_) => Opcodes::IMM_BORROW_FIELD_GENERIC,
        MutBorrowGlobal(_) => Opcodes::MUT_BORROW_GLOBAL,
        MutBorrowGlobalGeneric(_) => Opcodes::MUT_BORROW_GLOBAL_GENERIC,
        ImmBorrowGlobal(_) => Opcodes::IMM_BORROW_GLOBAL,
        ImmBorrowGlobalGeneric(_) => Opcodes::IMM_BORROW_GLOBAL_GENERIC,
        Add => Opcodes::ADD,
        Sub => Opcodes::SUB,
        Mul => Opcodes::MUL,
        Mod => Opcodes::MOD,
        Div => Opcodes::DIV,
        BitOr => Opcodes::BIT_OR,
        BitAnd => Opcodes::BIT_AND,
        Xor => Opcodes::XOR,
        Shl => Opcodes::SHL,
        Shr => Opcodes::SHR,
        Or => Opcodes::OR,
        And => Opcodes::AND,
        Not => Opcodes::NOT,
        Eq => Opcodes::EQ,
        Neq => Opcodes::NEQ,
        Lt => Opcodes::LT,
        Gt => Opcodes::GT,
        Le => Opcodes::LE,
        Ge => Opcodes::GE,
        Abort => Opcodes::ABORT,
        GetTxnGasUnitPrice => Opcodes::GET_TXN_GAS_UNIT_PRICE,
        GetTxnMaxGasUnits => Opcodes::GET_TXN_MAX_GAS_UNITS,
        GetGasRemaining => Opcodes::GET_GAS_REMAINING,
        GetTxnSenderAddress => Opcodes::GET_TXN_SENDER,
        Exists(_) => Opcodes::EXISTS,
        ExistsGeneric(_) => Opcodes::EXISTS_GENERIC,
        MoveFrom(_) => Opcodes::MOVE_FROM,
        MoveFromGeneric(_) => Opcodes::MOVE_FROM_GENERIC,
        MoveToSender(_) => Opcodes::MOVE_TO,
        MoveToSenderGeneric(_) => Opcodes::MOVE_TO_GENERIC,
        GetTxnSequenceNumber => Opcodes::GET_TXN_SEQUENCE_NUMBER,
        GetTxnPublicKey => Opcodes::GET_TXN_PUBLIC_KEY,
        Nop => Opcodes::NOP,
    };
    opcode as u8
}

/// The cost tables, keyed by the serialized form of the bytecode instruction.  We use the
/// serialized form as opposed to the instruction enum itself as the key since this will be the
/// on-chain representation of bytecode instructions in the future.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostTable {
    pub instruction_table: Vec<GasCost>,
    pub native_table: Vec<GasCost>,
}

impl CostTable {
    pub fn new(mut instrs: Vec<(Bytecode, GasCost)>, native_table: Vec<GasCost>) -> Self {
        instrs.sort_by_key(|cost| instruction_key(&cost.0));

        if cfg!(debug_assertions) {
            let mut instructions_covered = 0;
            for (index, (instr, _)) in instrs.iter().enumerate() {
                let key = instruction_key(instr);
                if index == (key - 1) as usize {
                    instructions_covered += 1;
                }
            }
            debug_assert!(
                instructions_covered == Bytecode::NUM_INSTRUCTIONS,
                "all instructions must be in the cost table"
            );
        }

        let instruction_table = instrs
            .into_iter()
            .map(|(_, cost)| cost)
            .collect::<Vec<GasCost>>();
        Self {
            instruction_table,
            native_table,
        }
    }

    #[inline]
    pub fn instruction_cost(&self, instr_index: u8) -> &GasCost {
        precondition!(instr_index > 0 && instr_index <= (self.instruction_table.len() as u8));
        &self.instruction_table[(instr_index - 1) as usize]
    }

    #[inline]
    pub fn native_cost(&self, native_index: NativeCostIndex) -> &GasCost {
        precondition!(
            native_index as u8 > 0 && native_index as u8 <= (self.instruction_table.len() as u8)
        );
        &self.native_table[native_index as usize]
    }

    pub fn get_gas(
        &self,
        instr: &Bytecode,
        size_provider: AbstractMemorySize<GasCarrier>,
    ) -> GasCost {
        // NB: instruction keys are 1-indexed. This means that their location in the cost array
        // will be the key - 1.
        let key = instruction_key(instr);
        let cost = self.instruction_table.get((key - 1) as usize);
        assume!(cost.is_some());
        let good_cost = cost.unwrap();
        GasCost {
            instruction_gas: good_cost.instruction_gas.map2(size_provider, Mul::mul),
            memory_gas: good_cost.memory_gas.map2(size_provider, Mul::mul),
        }
    }

    // Only used for genesis and for tests where we need a cost table and
    // don't have a genesis storage state.
    pub fn zero() -> Self {
        use Bytecode::*;
        // The actual costs for the instructions in this table _DO NOT MATTER_. This is only used
        // for genesis and testing, and for these cases we don't need to worry
        // about the actual gas for instructions.  The only thing we care about is having an entry
        // in the gas schedule for each instruction.
        let instrs = vec![
            (
                MoveToSender(StructDefinitionIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (
                MoveToSenderGeneric(StructDefInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (GetTxnSenderAddress, GasCost::new(0, 0)),
            (MoveFrom(StructDefinitionIndex::new(0)), GasCost::new(0, 0)),
            (
                MoveFromGeneric(StructDefInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (BrTrue(0), GasCost::new(0, 0)),
            (WriteRef, GasCost::new(0, 0)),
            (Mul, GasCost::new(0, 0)),
            (MoveLoc(0), GasCost::new(0, 0)),
            (And, GasCost::new(0, 0)),
            (GetTxnPublicKey, GasCost::new(0, 0)),
            (Pop, GasCost::new(0, 0)),
            (BitAnd, GasCost::new(0, 0)),
            (ReadRef, GasCost::new(0, 0)),
            (Sub, GasCost::new(0, 0)),
            (MutBorrowField(FieldHandleIndex::new(0)), GasCost::new(0, 0)),
            (
                MutBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (ImmBorrowField(FieldHandleIndex::new(0)), GasCost::new(0, 0)),
            (
                ImmBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (Add, GasCost::new(0, 0)),
            (CopyLoc(0), GasCost::new(0, 0)),
            (StLoc(0), GasCost::new(0, 0)),
            (Ret, GasCost::new(0, 0)),
            (Lt, GasCost::new(0, 0)),
            (LdU8(0), GasCost::new(0, 0)),
            (LdU64(0), GasCost::new(0, 0)),
            (LdU128(0), GasCost::new(0, 0)),
            (CastU8, GasCost::new(0, 0)),
            (CastU64, GasCost::new(0, 0)),
            (CastU128, GasCost::new(0, 0)),
            (Abort, GasCost::new(0, 0)),
            (MutBorrowLoc(0), GasCost::new(0, 0)),
            (ImmBorrowLoc(0), GasCost::new(0, 0)),
            (LdAddr(AddressPoolIndex::new(0)), GasCost::new(0, 0)),
            (Ge, GasCost::new(0, 0)),
            (Xor, GasCost::new(0, 0)),
            (Shl, GasCost::new(0, 0)),
            (Shr, GasCost::new(0, 0)),
            (Neq, GasCost::new(0, 0)),
            (Not, GasCost::new(0, 0)),
            (Call(FunctionHandleIndex::new(0)), GasCost::new(0, 0)),
            (
                CallGeneric(FunctionInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (Le, GasCost::new(0, 0)),
            (Branch(0), GasCost::new(0, 0)),
            (Unpack(StructDefinitionIndex::new(0)), GasCost::new(0, 0)),
            (
                UnpackGeneric(StructDefInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (Or, GasCost::new(0, 0)),
            (LdFalse, GasCost::new(0, 0)),
            (LdTrue, GasCost::new(0, 0)),
            (GetTxnGasUnitPrice, GasCost::new(0, 0)),
            (Mod, GasCost::new(0, 0)),
            (BrFalse(0), GasCost::new(0, 0)),
            (Exists(StructDefinitionIndex::new(0)), GasCost::new(0, 0)),
            (
                ExistsGeneric(StructDefInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (GetGasRemaining, GasCost::new(0, 0)),
            (BitOr, GasCost::new(0, 0)),
            (GetTxnMaxGasUnits, GasCost::new(0, 0)),
            (GetTxnSequenceNumber, GasCost::new(0, 0)),
            (FreezeRef, GasCost::new(0, 0)),
            (
                MutBorrowGlobal(StructDefinitionIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (
                MutBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (
                ImmBorrowGlobal(StructDefinitionIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (
                ImmBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (Div, GasCost::new(0, 0)),
            (Eq, GasCost::new(0, 0)),
            (LdByteArray(ByteArrayPoolIndex::new(0)), GasCost::new(0, 0)),
            (Gt, GasCost::new(0, 0)),
            (Pack(StructDefinitionIndex::new(0)), GasCost::new(0, 0)),
            (
                PackGeneric(StructDefInstantiationIndex::new(0)),
                GasCost::new(0, 0),
            ),
            (Nop, GasCost::new(0, 0)),
        ];
        let native_table = (0..NUMBER_OF_NATIVE_FUNCTIONS)
            .map(|_| GasCost::new(0, 0))
            .collect::<Vec<GasCost>>();
        CostTable::new(instrs, native_table)
    }
}

/// The  `GasCost` tracks:
/// - instruction cost: how much time/computational power is needed to perform the instruction
/// - memory cost: how much memory is required for the instruction, and storage overhead
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasCost {
    pub instruction_gas: GasUnits<GasCarrier>,
    pub memory_gas: GasUnits<GasCarrier>,
}

impl GasCost {
    pub fn new(instr_gas: GasCarrier, mem_gas: GasCarrier) -> Self {
        Self {
            instruction_gas: GasUnits::new(instr_gas),
            memory_gas: GasUnits::new(mem_gas),
        }
    }

    /// Take a GasCost from our gas schedule and convert it to a total gas charge in `GasUnits`.
    ///
    /// This is used internally for converting from a `GasCost` which is a triple of numbers
    /// represeing instruction, stack, and memory consumption into a number of `GasUnits`.
    #[inline]
    pub fn total(&self) -> GasUnits<GasCarrier> {
        self.instruction_gas.add(self.memory_gas)
    }
}

/// Computes the number of words rounded up
pub fn words_in(size: AbstractMemorySize<GasCarrier>) -> AbstractMemorySize<GasCarrier> {
    precondition!(size.get() <= MAX_ABSTRACT_MEMORY_SIZE.get() - (WORD_SIZE.get() + 1));
    // round-up div truncate
    size.map2(WORD_SIZE, |size, word_size| {
        // static invariant
        assume!(word_size > 0);
        // follows from the precondition
        assume!(size <= u64::max_value() - word_size);
        (size + (word_size - 1)) / word_size
    })
}

/// Calculate the intrinsic gas for the transaction based upon its size in bytes/words.
pub fn calculate_intrinsic_gas(
    transaction_size: AbstractMemorySize<GasCarrier>,
) -> GasUnits<GasCarrier> {
    precondition!(transaction_size.get() <= MAX_TRANSACTION_SIZE_IN_BYTES as GasCarrier);
    let min_transaction_fee = MIN_TRANSACTION_GAS_UNITS;

    if transaction_size.get() > LARGE_TRANSACTION_CUTOFF.get() {
        let excess = words_in(transaction_size.sub(LARGE_TRANSACTION_CUTOFF));
        min_transaction_fee.add(INTRINSIC_GAS_PER_BYTE.mul(excess))
    } else {
        min_transaction_fee.unitary_cast()
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum NativeCostIndex {
    SHA2_256 = 0,
    SHA3_256 = 1,
    ED25519_VERIFY = 2,
    ED25519_THRESHOLD_VERIFY = 3,
    ADDRESS_TO_BYTES = 4,
    U64_TO_BYTES = 5,
    BYTEARRAY_CONCAT = 6,
    LENGTH = 7,
    EMPTY = 8,
    BORROW = 9,
    BORROW_MUT = 10,
    PUSH_BACK = 11,
    POP_BACK = 12,
    DESTROY_EMPTY = 13,
    SWAP = 14,
    WRITE_TO_EVENT_STORE = 15,
    SAVE_ACCOUNT = 16,
}
