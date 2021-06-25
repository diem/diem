// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::prelude::*;
use move_binary_format::errors::VMError;
use move_core_types::vm_status::{known_locations, StatusCode, VMStatus};
use move_vm_runtime::logging::LogContext;

/// Error codes that can be emitted by the prologue. These have special significance to the VM when
/// they are raised during the prologue.
/// These errors are only expected from the `known_locations::account_module_abort()` module
/// The prologue should not emit any other error codes or fail for any reason, doing so will result
/// in the VM throwing an invariant violation
pub const EACCOUNT_FROZEN: u64 = 1000; // sending account is frozen
pub const EBAD_ACCOUNT_AUTHENTICATION_KEY: u64 = 1001; // auth key in transaction is invalid
pub const ESEQUENCE_NUMBER_TOO_OLD: u64 = 1002; // transaction sequence number is too old
pub const ESEQUENCE_NUMBER_TOO_NEW: u64 = 1003; // transaction sequence number is too new
pub const EACCOUNT_DOES_NOT_EXIST: u64 = 1004; // transaction sender's account does not exist
pub const ECANT_PAY_GAS_DEPOSIT: u64 = 1005; // insufficient balance (to pay for gas deposit)
pub const ETRANSACTION_EXPIRED: u64 = 1006; // transaction expiration time exceeds block time.
pub const EBAD_CHAIN_ID: u64 = 1007; // chain_id in transaction doesn't match the one on-chain
pub const ESCRIPT_NOT_ALLOWED: u64 = 1008;
pub const EMODULE_NOT_ALLOWED: u64 = 1009;
pub const EINVALID_WRITESET_SENDER: u64 = 1010; // invalid sender (not diem root) for write set
pub const ESEQUENCE_NUMBER_TOO_BIG: u64 = 1011;
pub const EBAD_TRANSACTION_FEE_CURRENCY: u64 = 1012;
pub const ESECONDARY_KEYS_ADDRESSES_COUNT_MISMATCH: u64 = 1013;
pub const ESEQ_NONCE_NONCE_INVALID: u64 = 1014;

const INVALID_STATE: u8 = 1;
const INVALID_ARGUMENT: u8 = 7;
const LIMIT_EXCEEDED: u8 = 8;

fn error_split(code: u64) -> (u8, u64) {
    let category = code as u8;
    let reason = code >> 8;
    (category, reason)
}

/// Converts particular Move abort codes to specific validation error codes for the prologue
/// Any non-abort non-execution code is considered an invariant violation, specifically
/// `UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION`
pub fn convert_prologue_error(
    error: VMError,
    log_context: &impl LogContext,
) -> Result<(), VMStatus> {
    let status = error.into_vm_status();
    Err(match status {
        VMStatus::Executed => VMStatus::Executed,
        VMStatus::MoveAbort(location, code)
            if location != known_locations::account_module_abort() =>
        {
            let (category, reason) = error_split(code);
            log_context.alert();
            error!(
                *log_context,
                "[diem_vm] Unexpected prologue Move abort: {:?}::{:?} (Category: {:?} Reason: {:?})",
                location, code, category, reason,
            );
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
        VMStatus::MoveAbort(location, code) => {
            let new_major_status = match error_split(code) {
                (INVALID_STATE, EACCOUNT_FROZEN) => StatusCode::SENDING_ACCOUNT_FROZEN,
                // Invalid authentication key
                (INVALID_ARGUMENT, EBAD_ACCOUNT_AUTHENTICATION_KEY) => StatusCode::INVALID_AUTH_KEY,
                // Sequence number too old
                (INVALID_ARGUMENT, ESEQUENCE_NUMBER_TOO_OLD) => StatusCode::SEQUENCE_NUMBER_TOO_OLD,
                // Sequence number too new
                (INVALID_ARGUMENT, ESEQUENCE_NUMBER_TOO_NEW) => StatusCode::SEQUENCE_NUMBER_TOO_NEW,
                // Sequence number too new
                (INVALID_ARGUMENT, EACCOUNT_DOES_NOT_EXIST) => {
                    StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
                }
                // Can't pay for transaction gas deposit/fee
                (INVALID_ARGUMENT, ECANT_PAY_GAS_DEPOSIT) => {
                    StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE
                }
                (INVALID_ARGUMENT, ETRANSACTION_EXPIRED) => StatusCode::TRANSACTION_EXPIRED,
                (INVALID_ARGUMENT, EBAD_CHAIN_ID) => StatusCode::BAD_CHAIN_ID,
                (INVALID_STATE, ESCRIPT_NOT_ALLOWED) => StatusCode::UNKNOWN_SCRIPT,
                (INVALID_STATE, EMODULE_NOT_ALLOWED) => StatusCode::INVALID_MODULE_PUBLISHER,
                (INVALID_ARGUMENT, EINVALID_WRITESET_SENDER) => StatusCode::REJECTED_WRITE_SET,
                // Sequence number will overflow
                (LIMIT_EXCEEDED, ESEQUENCE_NUMBER_TOO_BIG) => StatusCode::SEQUENCE_NUMBER_TOO_BIG,
                // The gas currency is not registered as a TransactionFee currency
                (INVALID_ARGUMENT, EBAD_TRANSACTION_FEE_CURRENCY) => {
                    StatusCode::BAD_TRANSACTION_FEE_CURRENCY
                }
                (INVALID_ARGUMENT, ESECONDARY_KEYS_ADDRESSES_COUNT_MISMATCH) => {
                    StatusCode::SECONDARY_KEYS_ADDRESSES_COUNT_MISMATCH
                }
                (INVALID_ARGUMENT, ESEQ_NONCE_NONCE_INVALID) => StatusCode::SEQUENCE_NONCE_INVALID,
                (category, reason) => {
                    log_context.alert();
                    error!(
                        *log_context,
                        "[diem_vm] Unexpected prologue Move abort: {:?}::{:?} (Category: {:?} Reason: {:?})",
                        location, code, category, reason,
                    );
                    return Err(VMStatus::Error(
                        StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION,
                    ));
                }
            };
            VMStatus::Error(new_major_status)
        }
        status @ VMStatus::ExecutionFailure { .. } | status @ VMStatus::Error(_) => {
            log_context.alert();
            error!(
                *log_context,
                "[diem_vm] Unexpected prologue error: {:?}", status
            );
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
    })
}

/// Checks for only Move aborts or successful execution.
/// Any other errors are mapped to the invariant violation
/// `UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION`
pub fn convert_epilogue_error(
    error: VMError,
    log_context: &impl LogContext,
) -> Result<(), VMStatus> {
    let status = error.into_vm_status();
    Err(match status {
        VMStatus::Executed => VMStatus::Executed,
        VMStatus::MoveAbort(location, code)
            if location != known_locations::account_module_abort() =>
        {
            let (category, reason) = error_split(code);
            log_context.alert();
            error!(
                *log_context,
                "[diem_vm] Unexpected success epilogue Move abort: {:?}::{:?} (Category: {:?} Reason: {:?})",
                location, code, category, reason,
            );
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }

        VMStatus::MoveAbort(location, code) => match error_split(code) {
            (LIMIT_EXCEEDED, ECANT_PAY_GAS_DEPOSIT) => VMStatus::MoveAbort(location, code),
            (category, reason) => {
                log_context.alert();
                error!(
                    *log_context,
                    "[diem_vm] Unexpected success epilogue Move abort: {:?}::{:?} (Category: {:?} Reason: {:?})",
                    location, code, category, reason,
                );
                VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
            }
        },

        status => {
            log_context.alert();
            error!(
                *log_context,
                "[diem_vm] Unexpected success epilogue error: {:?}", status,
            );
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
    })
}

/// Checks for only successful execution
/// Any errors are mapped to the invariant violation
/// `UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION`
pub fn expect_only_successful_execution(
    error: VMError,
    function_name: &str,
    log_context: &impl LogContext,
) -> Result<(), VMStatus> {
    let status = error.into_vm_status();
    Err(match status {
        VMStatus::Executed => VMStatus::Executed,

        status => {
            log_context.alert();
            error!(
                *log_context,
                "[diem_vm] Unexpected error from known Move function, '{}'. Error: {:?}",
                function_name,
                status,
            );
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
    })
}
