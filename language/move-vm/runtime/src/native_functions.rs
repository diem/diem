// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::{account_address::AccountAddress, account_config::CORE_CODE_ADDRESS};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    native_functions::{
        account, context::NativeContext, dispatch::NativeResult, event, hash, lcs, signature,
    },
    values::{debug, vector, Value},
};
use std::collections::VecDeque;
use vm::errors::VMResult;

/// The set of native functions the VM supports.
/// The functions can leave in any crate linked in but the VM declares them here.
/// 2 functions have to be implemented for a `NativeFunction`:
/// - `resolve` which given a function unique name ModuleAddress::ModuleName::FunctionName
/// returns a `NativeFunction`
/// - `dispatch` which given a `NativeFunction` invokes the native
#[derive(Debug, Clone, Copy)]
pub(crate) enum NativeFunction {
    HashSha2_256,
    HashSha3_256,
    LCSToBytes,
    SigED25519Verify,
    SigED25519ThresholdVerify,
    VectorLength,
    VectorEmpty,
    VectorBorrow,
    VectorBorrowMut,
    VectorPushBack,
    VectorPopBack,
    VectorDestroyEmpty,
    VectorSwap,
    AccountWriteEvent,
    AccountSaveAccount,
    DebugPrint,
    DebugPrintStackTrace,
}

// TODO: maybe we should go back to the single NativeFunction impl
pub(crate) struct FunctionResolver();

impl FunctionResolver {
    pub fn resolve(
        module_address: &AccountAddress,
        module_name: &str,
        function_name: &str,
    ) -> Option<NativeFunction> {
        use NativeFunction::*;

        let case = (module_address, module_name, function_name);
        Some(match case {
            (&CORE_CODE_ADDRESS, "Hash", "sha2_256") => HashSha2_256,
            (&CORE_CODE_ADDRESS, "Hash", "sha3_256") => HashSha3_256,
            (&CORE_CODE_ADDRESS, "LCS", "to_bytes") => LCSToBytes,
            (&CORE_CODE_ADDRESS, "Signature", "ed25519_verify") => SigED25519Verify,
            (&CORE_CODE_ADDRESS, "Signature", "ed25519_threshold_verify") => {
                SigED25519ThresholdVerify
            }
            (&CORE_CODE_ADDRESS, "Vector", "length") => VectorLength,
            (&CORE_CODE_ADDRESS, "Vector", "empty") => VectorEmpty,
            (&CORE_CODE_ADDRESS, "Vector", "borrow") => VectorBorrow,
            (&CORE_CODE_ADDRESS, "Vector", "borrow_mut") => VectorBorrowMut,
            (&CORE_CODE_ADDRESS, "Vector", "push_back") => VectorPushBack,
            (&CORE_CODE_ADDRESS, "Vector", "pop_back") => VectorPopBack,
            (&CORE_CODE_ADDRESS, "Vector", "destroy_empty") => VectorDestroyEmpty,
            (&CORE_CODE_ADDRESS, "Vector", "swap") => VectorSwap,
            (&CORE_CODE_ADDRESS, "Event", "write_to_event_store") => AccountWriteEvent,
            (&CORE_CODE_ADDRESS, "LibraAccount", "save_account") => AccountSaveAccount,
            (&CORE_CODE_ADDRESS, "Debug", "print") => DebugPrint,
            (&CORE_CODE_ADDRESS, "Debug", "print_stack_trace") => DebugPrintStackTrace,
            _ => return None,
        })
    }
}

impl NativeFunction {
    /// Given the vector of aguments, it executes the native function.
    pub(crate) fn dispatch(
        self,
        ctx: &mut impl NativeContext,
        t: Vec<Type>,
        v: VecDeque<Value>,
    ) -> VMResult<NativeResult> {
        match self {
            Self::HashSha2_256 => hash::native_sha2_256(ctx, t, v),
            Self::HashSha3_256 => hash::native_sha3_256(ctx, t, v),
            Self::SigED25519Verify => signature::native_ed25519_signature_verification(ctx, t, v),
            Self::SigED25519ThresholdVerify => {
                signature::native_ed25519_threshold_signature_verification(ctx, t, v)
            }
            Self::VectorLength => vector::native_length(ctx, t, v),
            Self::VectorEmpty => vector::native_empty(ctx, t, v),
            Self::VectorBorrow => vector::native_borrow(ctx, t, v),
            Self::VectorBorrowMut => vector::native_borrow(ctx, t, v),
            Self::VectorPushBack => vector::native_push_back(ctx, t, v),
            Self::VectorPopBack => vector::native_pop(ctx, t, v),
            Self::VectorDestroyEmpty => vector::native_destroy_empty(ctx, t, v),
            Self::VectorSwap => vector::native_swap(ctx, t, v),
            Self::AccountWriteEvent => event::native_emit_event(ctx, t, v),
            Self::AccountSaveAccount => account::native_save_account(ctx, t, v),
            Self::LCSToBytes => lcs::native_to_bytes(ctx, t, v),
            Self::DebugPrint => debug::native_print(ctx, t, v),
            Self::DebugPrintStackTrace => debug::native_print_stack_trace(ctx, t, v),
        }
    }
}
