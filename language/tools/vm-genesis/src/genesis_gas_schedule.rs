// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file contains the starting gas schedule published at genesis.

use libra_vm::system_module_names::GAS_SCHEDULE_MODULE;
use move_vm_runtime::MoveVM;
use move_vm_state::{data_cache::RemoteCache, execution_context::TransactionExecutionContext};
use move_vm_types::{loaded_data::types::Type, values::Value};
use once_cell::sync::Lazy;
use vm::{
    file_format::{
        Bytecode, ConstantPoolIndex, FieldHandleIndex, FieldInstantiationIndex,
        FunctionHandleIndex, FunctionInstantiationIndex, StructDefInstantiationIndex,
        StructDefinitionIndex, NUMBER_OF_NATIVE_FUNCTIONS,
    },
    gas_schedule::{CostTable, GasCost, GAS_SCHEDULE_NAME, MAXIMUM_NUMBER_OF_GAS_UNITS},
};

static INITIAL_GAS_SCHEDULE: Lazy<Vec<u8>> = Lazy::new(|| {
    use Bytecode::*;
    let instrs = vec![
        (
            MoveToSender(StructDefinitionIndex::new(0)),
            GasCost::new(774, 1),
        ),
        (
            MoveToSenderGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(774, 1),
        ),
        (GetTxnSenderAddress, GasCost::new(30, 1)),
        (
            MoveFrom(StructDefinitionIndex::new(0)),
            GasCost::new(917, 1),
        ),
        (
            MoveFromGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(917, 1),
        ),
        (BrTrue(0), GasCost::new(31, 1)),
        (WriteRef, GasCost::new(65, 1)),
        (Mul, GasCost::new(41, 1)),
        (MoveLoc(0), GasCost::new(41, 1)),
        (And, GasCost::new(49, 1)),
        (GetTxnPublicKey, GasCost::new(41, 1)),
        (Pop, GasCost::new(27, 1)),
        (BitAnd, GasCost::new(44, 1)),
        (ReadRef, GasCost::new(51, 1)),
        (Sub, GasCost::new(44, 1)),
        (
            MutBorrowField(FieldHandleIndex::new(0)),
            GasCost::new(58, 1),
        ),
        (
            MutBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
            GasCost::new(58, 1),
        ),
        (
            ImmBorrowField(FieldHandleIndex::new(0)),
            GasCost::new(58, 1),
        ),
        (
            ImmBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
            GasCost::new(58, 1),
        ),
        (Add, GasCost::new(45, 1)),
        (CopyLoc(0), GasCost::new(41, 1)),
        (StLoc(0), GasCost::new(28, 1)),
        (Ret, GasCost::new(28, 1)),
        (Lt, GasCost::new(49, 1)),
        (LdU8(0), GasCost::new(29, 1)),
        (LdU64(0), GasCost::new(29, 1)),
        (LdU128(0), GasCost::new(29, 1)),
        (CastU8, GasCost::new(29, 1)),
        (CastU64, GasCost::new(29, 1)),
        (CastU128, GasCost::new(29, 1)),
        (Abort, GasCost::new(39, 1)),
        (MutBorrowLoc(0), GasCost::new(45, 1)),
        (ImmBorrowLoc(0), GasCost::new(45, 1)),
        (LdConst(ConstantPoolIndex::new(0)), GasCost::new(36, 1)),
        (Ge, GasCost::new(46, 1)),
        (Xor, GasCost::new(46, 1)),
        (Shl, GasCost::new(46, 1)),
        (Shr, GasCost::new(46, 1)),
        (Neq, GasCost::new(51, 1)),
        (Not, GasCost::new(35, 1)),
        (Call(FunctionHandleIndex::new(0)), GasCost::new(197, 1)),
        (
            CallGeneric(FunctionInstantiationIndex::new(0)),
            GasCost::new(197, 1),
        ),
        (Le, GasCost::new(47, 1)),
        (Branch(0), GasCost::new(10, 1)),
        (Unpack(StructDefinitionIndex::new(0)), GasCost::new(94, 1)),
        (
            UnpackGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(94, 1),
        ),
        (Or, GasCost::new(43, 1)),
        (LdFalse, GasCost::new(30, 1)),
        (LdTrue, GasCost::new(29, 1)),
        (GetTxnGasUnitPrice, GasCost::new(29, 1)),
        (Mod, GasCost::new(42, 1)),
        (BrFalse(0), GasCost::new(29, 1)),
        (Exists(StructDefinitionIndex::new(0)), GasCost::new(856, 1)),
        (
            ExistsGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(856, 1),
        ),
        (GetGasRemaining, GasCost::new(32, 1)),
        (BitOr, GasCost::new(45, 1)),
        (GetTxnMaxGasUnits, GasCost::new(34, 1)),
        (GetTxnSequenceNumber, GasCost::new(29, 1)),
        (FreezeRef, GasCost::new(10, 1)),
        (
            MutBorrowGlobal(StructDefinitionIndex::new(0)),
            GasCost::new(929, 1),
        ),
        (
            MutBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(929, 1),
        ),
        (
            ImmBorrowGlobal(StructDefinitionIndex::new(0)),
            GasCost::new(929, 1),
        ),
        (
            ImmBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(929, 1),
        ),
        (Div, GasCost::new(41, 1)),
        (Eq, GasCost::new(48, 1)),
        (Gt, GasCost::new(46, 1)),
        (Pack(StructDefinitionIndex::new(0)), GasCost::new(73, 1)),
        (
            PackGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(73, 1),
        ),
        (Nop, GasCost::new(10, 1)),
    ];
    // TODO Zero for now, this is going to be filled in later
    let native_table = (0..NUMBER_OF_NATIVE_FUNCTIONS)
        .map(|_| GasCost::new(0, 0))
        .collect::<Vec<GasCost>>();
    let cost_table = CostTable::new(instrs, native_table);
    lcs::to_bytes(&cost_table).expect("Unable to serialize genesis gas schedule for instructions")
});

pub(crate) fn initial_gas_schedule(move_vm: &MoveVM, data_view: &dyn RemoteCache) -> Value {
    let struct_ty = move_vm
        .resolve_struct_def_by_name(
            &GAS_SCHEDULE_MODULE,
            &GAS_SCHEDULE_NAME,
            &mut TransactionExecutionContext::new(MAXIMUM_NUMBER_OF_GAS_UNITS, data_view),
            &[],
        )
        .expect("GasSchedule Module must exist");
    Value::simple_deserialize(&INITIAL_GAS_SCHEDULE, Type::Struct(Box::new(struct_ty))).unwrap()
}
