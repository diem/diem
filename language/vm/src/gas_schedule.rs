// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module lays out the basic abstract costing schedule for bytecode instructions.
//!
//! It is important to note that the cost schedule defined in this file does not track hashing
//! operations or other native operations; the cost of each native operation will be returned by the
//! native function itself.
use crate::file_format::{
    Bytecode, ConstantPoolIndex, FieldHandleIndex, FieldInstantiationIndex, FunctionHandleIndex,
    FunctionInstantiationIndex, StructDefInstantiationIndex, StructDefinitionIndex,
    NUMBER_OF_NATIVE_FUNCTIONS,
};
pub use crate::file_format_common::Opcodes;
use libra_types::transaction::MAX_TRANSACTION_SIZE_IN_BYTES;
use move_core_types::{
    gas_schedule::{
        words_in, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasConstants, GasCost,
        GasUnits,
    },
    identifier::Identifier,
};
use once_cell::sync::Lazy;
use std::ops::Mul;

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
        LdConst(_) => Opcodes::LD_CONST,
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

pub fn get_gas(
    table: &CostTable,
    instr: &Bytecode,
    size_provider: AbstractMemorySize<GasCarrier>,
) -> GasCost {
    // NB: instruction keys are 1-indexed. This means that their location in the cost array
    // will be the key - 1.
    let key = instruction_key(instr);
    let cost = table.instruction_table.get((key - 1) as usize);
    assume!(cost.is_some());
    let good_cost = cost.unwrap();
    GasCost {
        instruction_gas: good_cost.instruction_gas.map2(size_provider, Mul::mul),
        memory_gas: good_cost.memory_gas.map2(size_provider, Mul::mul),
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
