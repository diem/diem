// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module lays out the basic abstract costing schedule for bytecode instructions.
//!
//! It is important to note that the cost schedule defined in this file does not track hashing
//! operations or other native operations; the cost of each native operation will be returned by the
//! native function itself.
use libra_types::{
    transaction::MAX_TRANSACTION_SIZE_IN_BYTES,
    vm_error::{StatusCode, VMStatus},
};
use mirai_annotations::*;
use move_core_types::gas_schedule::{
    words_in, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasConstants, GasCost,
    GasUnits,
};
use vm::{
    errors::VMResult,
    file_format::{
        Bytecode, ConstantPoolIndex, FieldHandleIndex, FieldInstantiationIndex,
        FunctionHandleIndex, FunctionInstantiationIndex, StructDefInstantiationIndex,
        StructDefinitionIndex, NUMBER_OF_NATIVE_FUNCTIONS,
    },
    file_format_common::{instruction_key, Opcodes},
};

/// The Move VM implementation for gas charging.
///
/// Initialize with a `CostTable` and the gas provided to the transaction.
/// Provide all the proper guarantees about gas charging in the Move VM.
///
/// Every client must use an instance of this type to interact with the Move VM.
pub struct CostStrategy<'a> {
    cost_table: &'a CostTable,
    gas_left: GasUnits<GasCarrier>,
    charge: bool,
}

impl<'a> CostStrategy<'a> {
    /// A transaction `CostStrategy`. Charge for every operation and fails once there
    /// is no more gas to pay for operations.
    ///
    /// This is the instantiation the must be used when execution a user script.
    pub fn transaction(cost_table: &'a CostTable, gas_left: GasUnits<GasCarrier>) -> Self {
        Self {
            cost_table,
            gas_left,
            charge: true,
        }
    }

    /// A system `CostStrategy` does not charge for operations.
    ///
    /// It should be used by clients in very specific cases and when executing system
    /// code that does not have to charge the user.
    pub fn system(cost_table: &'a CostTable, gas_left: GasUnits<GasCarrier>) -> Self {
        Self {
            cost_table,
            gas_left,
            charge: false,
        }
    }

    /// Return the `CostTable` behind this `CostStrategy`.
    pub fn cost_table(&self) -> &CostTable {
        self.cost_table
    }

    /// Return the gas left.
    pub fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.gas_left
    }

    /// Charge a given amount of gas and fail if not enough gas units are left.
    pub fn deduct_gas(&mut self, amount: GasUnits<GasCarrier>) -> VMResult<()> {
        if !self.charge {
            return Ok(());
        }
        if self
            .gas_left
            .app(&amount, |curr_gas, gas_amt| curr_gas >= gas_amt)
        {
            self.gas_left = self.gas_left.sub(amount);
            Ok(())
        } else {
            // Zero out the internal gas state
            self.gas_left = GasUnits::new(0);
            Err(VMStatus::new(StatusCode::OUT_OF_GAS))
        }
    }

    /// Charge an instruction over data with a given size and fail if not enough gas units are left.
    pub fn charge_instr_with_size(
        &mut self,
        opcode: Opcodes,
        size: AbstractMemorySize<GasCarrier>,
    ) -> VMResult<()> {
        self.deduct_gas(
            self.cost_table
                .instruction_cost(opcode as u8)
                .total()
                .mul(size),
        )
    }

    /// Charge an instruction and fail if not enough gas units are left.
    pub fn charge_instr(&mut self, opcode: Opcodes) -> VMResult<()> {
        self.deduct_gas(self.cost_table.instruction_cost(opcode as u8).total())
    }

    /// Charge gas related to the overall size of a transaction and fail if not enough
    /// gas units are left.
    pub fn charge_intrinsic_gas(&mut self, size: AbstractMemorySize<GasCarrier>) -> VMResult<()> {
        let cost = calculate_intrinsic_gas(size, &self.cost_table.gas_constants);
        self.deduct_gas(cost)
    }
}

pub fn new_from_instructions(
    mut instrs: Vec<(Bytecode, GasCost)>,
    native_table: Vec<GasCost>,
) -> CostTable {
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
    CostTable {
        instruction_table,
        native_table,
        gas_constants: GasConstants::default(),
    }
}

// Only used for genesis and for tests where we need a cost table and
// don't have a genesis storage state.
pub fn zero_cost_schedule() -> CostTable {
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
        (MoveTo(StructDefinitionIndex::new(0)), GasCost::new(0, 0)),
        (
            MoveToGeneric(StructDefInstantiationIndex::new(0)),
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
        (LdConst(ConstantPoolIndex::new(0)), GasCost::new(0, 0)),
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
        (Mod, GasCost::new(0, 0)),
        (BrFalse(0), GasCost::new(0, 0)),
        (Exists(StructDefinitionIndex::new(0)), GasCost::new(0, 0)),
        (
            ExistsGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(0, 0),
        ),
        (BitOr, GasCost::new(0, 0)),
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
    new_from_instructions(instrs, native_table)
}

/// Calculate the intrinsic gas for the transaction based upon its size in bytes/words.
pub fn calculate_intrinsic_gas(
    transaction_size: AbstractMemorySize<GasCarrier>,
    gas_constants: &GasConstants,
) -> GasUnits<GasCarrier> {
    precondition!(transaction_size.get() <= MAX_TRANSACTION_SIZE_IN_BYTES as GasCarrier);
    let min_transaction_fee = gas_constants.min_transaction_gas_units;

    if transaction_size.get() > gas_constants.large_transaction_cutoff.get() {
        let excess = words_in(transaction_size.sub(gas_constants.large_transaction_cutoff));
        min_transaction_fee.add(gas_constants.instrinsic_gas_per_byte.mul(excess))
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
    ED25519_VALIDATE_KEY = 14,
}
