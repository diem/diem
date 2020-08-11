// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::prelude::*;
use move_core_types::vm_status::{known_locations, StatusCode, VMStatus};

/// Error codes that can be emitted by the prologue. These have special significance to the VM when
/// they are raised during the prologue.
/// These errors are only expected from the `known_locations::account_module_abort()` module
/// The prologue should not emit any other error codes or fail for any reason, doing so will result
/// in the VM throwing an invariant violation
pub const EACCOUNT_FROZEN: u64 = 0; // sending account is frozen
pub const EBAD_ACCOUNT_AUTHENTICATION_KEY: u64 = 1; // auth key in transaction is invalid
pub const ESEQUENCE_NUMBER_TOO_OLD: u64 = 2; // transaction sequence number is too old
pub const ESEQUENCE_NUMBER_TOO_NEW: u64 = 3; // transaction sequence number is too new
pub const EACCOUNT_DOES_NOT_EXIST: u64 = 4; // transaction sender's account does not exist
pub const EINSUFFICIENT_BALANCE: u64 = 5; // insufficient balance (to pay for gas deposit)
pub const ETRANSACTION_EXPIRED: u64 = 6; // transaction expiration time exceeds block time.
pub const EBAD_CHAIN_ID: u64 = 7; // chain_id in transaction doesn't match the one on-chain

// invalid sender (not libra root) for write set
pub const EINVALID_WRITESET_SENDER: u64 = 33;
// writeset prologue sequence nubmer is too new
pub const EWS_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 11;

/// Converts particular Move abort codes to specific validation error codes for the prologue
/// Any non-abort non-execution code is considered an invariant violation, specifically
/// `UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION`
pub fn convert_normal_prologue_error(status: VMStatus) -> VMStatus {
    match status {
        VMStatus::Executed => VMStatus::Executed,
        VMStatus::MoveAbort(location, code) => {
            if location != known_locations::account_module_abort() {
                crit!(
                    "[libra_vm] Unexpected prologue move abort: {:?}::{:?}",
                    location,
                    code
                );
                return VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION);
            }

            let new_major_status = match code {
                EACCOUNT_FROZEN => StatusCode::SENDING_ACCOUNT_FROZEN,
                // Invalid authentication key
                EBAD_ACCOUNT_AUTHENTICATION_KEY => StatusCode::INVALID_AUTH_KEY,
                // Sequence number too old
                ESEQUENCE_NUMBER_TOO_OLD => StatusCode::SEQUENCE_NUMBER_TOO_OLD,
                // Sequence number too new
                ESEQUENCE_NUMBER_TOO_NEW => StatusCode::SEQUENCE_NUMBER_TOO_NEW,
                // Sequence number too new
                EACCOUNT_DOES_NOT_EXIST => StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST,
                // Can't pay for transaction gas deposit/fee
                EINSUFFICIENT_BALANCE => StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE,
                ETRANSACTION_EXPIRED => StatusCode::TRANSACTION_EXPIRED,
                EBAD_CHAIN_ID => StatusCode::BAD_CHAIN_ID,
                code => {
                    crit!(
                        "[libra_vm] Unexpected prologue move abort: {:?}::{:?}",
                        location,
                        code
                    );
                    return VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION);
                }
            };
            VMStatus::Error(new_major_status)
        }
        status @ VMStatus::ExecutionFailure { .. } | status @ VMStatus::Error(_) => {
            crit!("[libra_vm] Unexpected prologue error: {:?}", status);
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
    }
}

/// Checks for only move aborts or successful execution.
/// Any other errors are mapped to the invariant violation
/// `UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION`
pub fn convert_normal_success_epilogue_error(status: VMStatus) -> VMStatus {
    match status {
        VMStatus::MoveAbort(location, code @ EACCOUNT_FROZEN)
        | VMStatus::MoveAbort(location, code @ EINSUFFICIENT_BALANCE) => {
            if location != known_locations::account_module_abort() {
                crit!(
                    "[libra_vm] Unexpected success epilogue move abort: {:?}::{:?}",
                    location,
                    code
                );
                return VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION);
            }
            VMStatus::MoveAbort(location, code)
        }

        status @ VMStatus::Executed => status,

        status => {
            crit!("[libra_vm] Unexpected success epilogue error: {:?}", status);
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
    }
}

/// Converts move aborts or execution failures to `REJECTED_WRITE_SET`
/// Any other errors are mapped to the invariant violation
/// `UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION`
pub fn convert_write_set_prologue_error(status: VMStatus) -> VMStatus {
    match status {
        VMStatus::Executed => VMStatus::Executed,
        VMStatus::MoveAbort(location, code @ EINVALID_WRITESET_SENDER)
        | VMStatus::MoveAbort(location, code @ ESEQUENCE_NUMBER_TOO_OLD)
        | VMStatus::MoveAbort(location, code @ EWS_PROLOGUE_SEQUENCE_NUMBER_TOO_NEW)
        | VMStatus::MoveAbort(location, code @ EBAD_ACCOUNT_AUTHENTICATION_KEY) => {
            if location != known_locations::write_set_manager_module_abort() {
                crit!(
                    "[libra_vm] Unexpected write set prologue Move abort: {:?}::{:?}",
                    location,
                    code
                );
                return VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION);
            }
            VMStatus::Error(StatusCode::REJECTED_WRITE_SET)
        }

        status => {
            crit!(
                "[libra_vm] Unexpected write set prologue error: {:?}",
                status
            );
            VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
        }
    }
}

/// Checks for only successful execution
/// Any errors are mapped to the invariant violation
/// `UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION`
pub fn expect_only_successful_execution<'a>(
    function_name: &'a str,
) -> Box<dyn FnOnce(VMStatus) -> VMStatus + 'a> {
    Box::new(move |status: VMStatus| -> VMStatus {
        match status {
            VMStatus::Executed => VMStatus::Executed,

            status => {
                crit!(
                    "[libra_vm] Unexpected error from known move function, '{}'. Error: {:?}",
                    function_name,
                    status
                );
                VMStatus::Error(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION)
            }
        }
    })
}
