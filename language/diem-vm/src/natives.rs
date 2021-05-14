use move_vm_types::make_enum_for_natives;

use move_binary_format::errors::PartialVMResult;
use move_core_types::{account_address::AccountAddress, identifier::Identifier};
use move_vm_types::natives::function::{NativeContext, NativeFunction, NativeResult};
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use std::collections::VecDeque;

use diem_framework::natives::*;
use move_stdlib::natives::*;

make_enum_for_natives!(
    DiemNative = NativeAccountCreateSigner
        | NativeAccountDestroySigner
        | NativeSignatureEd25519ValidatePubkey
        | NativeSignatureEd25519Verify
        | NativeBCSToBytes
        | NativeDebugPrint
        | NativeDebugPrintStackTrace
        | NativeSignerBorrowAddress
        | NativeHashSha2_256
        | NativeHashSha3_256
        | NativeEventWriteToEventStore
        | NativeVectorBorrow
        | NativeVectorDestroyEmpty
        | NativeVectorEmpty
        | NativeVectorLength
        | NativeVectorPopBack
        | NativeVectorPushBack
        | NativeVectorSwap
);

pub fn diem_natives() -> Vec<(AccountAddress, Identifier, Identifier, DiemNative)> {
    diem_framework::natives::all_natives()
        .into_iter()
        .chain(move_stdlib::natives::all_natives().into_iter())
        .map(|(module_name, func_name, func)| {
            (
                diem_types::account_config::CORE_CODE_ADDRESS,
                Identifier::new(module_name).unwrap(),
                Identifier::new(func_name).unwrap(),
                func,
            )
        })
        .collect()
}
