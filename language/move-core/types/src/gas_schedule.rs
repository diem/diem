// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module lays out the basic abstract costing schedule for bytecode instructions.
//!
//! It is important to note that the cost schedule defined in this file does not track hashing
//! operations or other native operations; the cost of each native operation will be returned by the
//! native function itself.
use mirai_annotations::*;
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

/// Zero cost.
pub const ZERO_GAS_UNITS: GasUnits<GasCarrier> = GasUnits(0);

/// The maximum size representable by AbstractMemorySize
pub const MAX_ABSTRACT_MEMORY_SIZE: AbstractMemorySize<GasCarrier> =
    AbstractMemorySize(std::u64::MAX);

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

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct GasConstants {
    /// The cost per-byte written to global storage.
    /// TODO: Fill this in with a proper number once it's determined.
    pub global_memory_per_byte_cost: GasUnits<GasCarrier>,

    /// The cost per-byte written to storage.
    /// TODO: Fill this in with a proper number once it's determined.
    pub global_memory_per_byte_write_cost: GasUnits<GasCarrier>,

    /// We charge one unit of gas per-byte for the first 600 bytes
    pub min_transaction_gas_units: GasUnits<GasCarrier>,

    /// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
    pub large_transaction_cutoff: AbstractMemorySize<GasCarrier>,

    /// The units of gas that should be charged per byte for every transaction.
    pub instrinsic_gas_per_byte: GasUnits<GasCarrier>,

    /// 1 nanosecond should equal one unit of computational gas. We bound the maximum
    /// computational time of any given transaction at 10 milliseconds. We want this number and
    /// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
    ///         MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
    pub maximum_number_of_gas_units: GasUnits<GasCarrier>,

    /// The minimum gas price that a transaction can be submitted with.
    pub min_price_per_gas_unit: GasPrice<GasCarrier>,

    /// The maximum gas unit price that a transaction can be submitted with.
    pub max_price_per_gas_unit: GasPrice<GasCarrier>,

    pub max_transaction_size_in_bytes: u64,
}

impl Default for GasConstants {
    fn default() -> Self {
        Self {
            global_memory_per_byte_cost: GasUnits(8),
            global_memory_per_byte_write_cost: GasUnits(8),
            min_transaction_gas_units: GasUnits(600),
            large_transaction_cutoff: AbstractMemorySize(600),
            instrinsic_gas_per_byte: GasUnits(8),
            maximum_number_of_gas_units: GasUnits(1_000_000),
            min_price_per_gas_unit: GasPrice(0),
            max_price_per_gas_unit: GasPrice(10_000),
            max_transaction_size_in_bytes: 4096,
        }
    }
}
/// The cost tables, keyed by the serialized form of the bytecode instruction.  We use the
/// serialized form as opposed to the instruction enum itself as the key since this will be the
/// on-chain representation of bytecode instructions in the future.
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct CostTable {
    pub instruction_table: Vec<GasCost>,
    pub native_table: Vec<GasCost>,
    pub gas_constants: GasConstants,
}

impl CostTable {
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
}

/// The  `GasCost` tracks:
/// - instruction cost: how much time/computational power is needed to perform the instruction
/// - memory cost: how much memory is required for the instruction, and storage overhead
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum NativeCostIndex {
    SHA2_256 = 0,
    SHA3_256 = 1,
    ED25519_VERIFY = 2,
    ED25519_THRESHOLD_VERIFY = 3,
    LCS_TO_BYTES = 4,
    LENGTH = 5,
    EMPTY = 6,
    BORROW = 7,
    BORROW_MUT = 8,
    PUSH_BACK = 9,
    POP_BACK = 10,
    DESTROY_EMPTY = 11,
    SWAP = 12,
    SAVE_ACCOUNT = 13,
}
