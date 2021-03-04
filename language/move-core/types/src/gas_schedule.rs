// Copyright (c) The Diem Core Contributors
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

    /// Apply a function `f` of two arguments to the carrier. Since `f` is not an endomorphism, we
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
    doc: "Units of gas as seen by clients of the Move VM."
}

define_gas_unit! {
    name: InternalGasUnits,
    carrier: GasCarrier,
    doc: "Units of gas used within the Move VM, scaled for fine-grained accounting."
}

define_gas_unit! {
    name: GasPrice,
    carrier: GasCarrier,
    doc: "A newtype wrapper around the gas price for each unit of gas consumed."
}

/// One unit of gas
pub const ONE_GAS_UNIT: InternalGasUnits<GasCarrier> = InternalGasUnits(1);

/// The maximum size representable by AbstractMemorySize
pub const MAX_ABSTRACT_MEMORY_SIZE: AbstractMemorySize<GasCarrier> =
    AbstractMemorySize(std::u64::MAX);

/// The size in bytes for a non-string or address constant on the stack
pub const CONST_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(16);

/// The size in bytes for a reference on the stack
pub const REFERENCE_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(8);

/// The size of a struct in bytes
pub const STRUCT_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(2);

/// For V1 all accounts will be ~800 bytes
pub const DEFAULT_ACCOUNT_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(800);

/// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
pub const LARGE_TRANSACTION_CUTOFF: AbstractMemorySize<GasCarrier> = AbstractMemorySize(600);

/// For exists checks on data that doesn't exists this is the multiplier that is used.
pub const MIN_EXISTS_DATA_SIZE: AbstractMemorySize<GasCarrier> = AbstractMemorySize(100);

pub const MAX_TRANSACTION_SIZE_IN_BYTES: GasCarrier = 4096;

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct GasConstants {
    /// The cost per-byte read from global storage.
    pub global_memory_per_byte_cost: InternalGasUnits<GasCarrier>,

    /// The cost per-byte written to storage.
    pub global_memory_per_byte_write_cost: InternalGasUnits<GasCarrier>,

    /// The flat minimum amount of gas required for any transaction.
    /// Charged at the start of execution.
    pub min_transaction_gas_units: InternalGasUnits<GasCarrier>,

    /// Any transaction over this size will be charged an additional amount per byte.
    pub large_transaction_cutoff: AbstractMemorySize<GasCarrier>,

    /// The units of gas that to be charged per byte over the `large_transaction_cutoff` in addition to
    /// `min_transaction_gas_units` for transactions whose size exceeds `large_transaction_cutoff`.
    pub intrinsic_gas_per_byte: InternalGasUnits<GasCarrier>,

    /// ~5 microseconds should equal one unit of computational gas. We bound the maximum
    /// computational time of any given transaction at roughly 20 seconds. We want this number and
    /// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
    /// MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
    /// NB: The bound is set quite high since custom scripts aren't allowed except from predefined
    /// and vetted senders.
    pub maximum_number_of_gas_units: GasUnits<GasCarrier>,

    /// The minimum gas price that a transaction can be submitted with.
    pub min_price_per_gas_unit: GasPrice<GasCarrier>,

    /// The maximum gas unit price that a transaction can be submitted with.
    pub max_price_per_gas_unit: GasPrice<GasCarrier>,

    pub max_transaction_size_in_bytes: GasCarrier,

    pub gas_unit_scaling_factor: GasCarrier,
    pub default_account_size: AbstractMemorySize<GasCarrier>,
}

impl GasConstants {
    pub fn to_internal_units(&self, units: GasUnits<GasCarrier>) -> InternalGasUnits<GasCarrier> {
        InternalGasUnits::new(units.get() * self.gas_unit_scaling_factor)
    }

    pub fn to_external_units(&self, units: InternalGasUnits<GasCarrier>) -> GasUnits<GasCarrier> {
        GasUnits::new(units.get() / self.gas_unit_scaling_factor)
    }
}

impl Default for GasConstants {
    fn default() -> Self {
        Self {
            global_memory_per_byte_cost: InternalGasUnits(4),
            global_memory_per_byte_write_cost: InternalGasUnits(9),
            min_transaction_gas_units: InternalGasUnits(600),
            large_transaction_cutoff: LARGE_TRANSACTION_CUTOFF,
            intrinsic_gas_per_byte: InternalGasUnits(8),
            maximum_number_of_gas_units: GasUnits(4_000_000),
            min_price_per_gas_unit: GasPrice(0),
            max_price_per_gas_unit: GasPrice(10_000),
            max_transaction_size_in_bytes: MAX_TRANSACTION_SIZE_IN_BYTES,
            gas_unit_scaling_factor: 1000,
            default_account_size: DEFAULT_ACCOUNT_SIZE,
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
    pub fn native_cost(&self, native_index: u8) -> &GasCost {
        precondition!(native_index < (self.native_table.len() as u8));
        &self.native_table[native_index as usize]
    }
}

/// The  `GasCost` tracks:
/// - instruction cost: how much time/computational power is needed to perform the instruction
/// - memory cost: how much memory is required for the instruction, and storage overhead
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GasCost {
    pub instruction_gas: InternalGasUnits<GasCarrier>,
    pub memory_gas: InternalGasUnits<GasCarrier>,
}

impl GasCost {
    pub fn new(instr_gas: GasCarrier, mem_gas: GasCarrier) -> Self {
        Self {
            instruction_gas: InternalGasUnits::new(instr_gas),
            memory_gas: InternalGasUnits::new(mem_gas),
        }
    }

    /// Convert a GasCost to a total gas charge in `InternalGasUnits`.
    #[inline]
    pub fn total(&self) -> InternalGasUnits<GasCarrier> {
        self.instruction_gas.add(self.memory_gas)
    }
}
