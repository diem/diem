// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::IndexKind;
use libra_types::{
    account_address::AccountAddress,
    transaction::TransactionStatus,
    vm_error::{StatusCode, VMStatus},
};

// We may want to eventually move this into the VM runtime since it is a semantic decision that
// need to be made by the VM. But for now, this will reside here.
pub fn vm_result_to_transaction_status<T>(result: &VMResult<T>) -> TransactionStatus {
    // The decision as to whether or not a transaction should be dropped should be able to be
    // determined solely by the VMStatus. This then means that we can audit/verify any decisions
    // made by the VM externally on whether or not to discard or keep the transaction output by
    // inspecting the contained VMStatus.
    let vm_status = vm_status_of_result(result);
    vm_status.into()
}

// TODO: Fill in the details for Locations. Ideally it should be a unique handle into a function and
// a pc.
#[derive(Debug, Default)]
pub struct Location {}

/// Error codes that can be emitted by the prologue. These have special significance to the VM when
/// they are raised during the prologue. However, they can also be raised by user code during
/// execution of a transaction script. They have no significance to the VM in that case.
pub const EBAD_SIGNATURE: u64 = 1; // signature on transaction is invalid
pub const EBAD_ACCOUNT_AUTHENTICATION_KEY: u64 = 2; // auth key in transaction is invalid
pub const ESEQUENCE_NUMBER_TOO_OLD: u64 = 3; // transaction sequence number is too old
pub const ESEQUENCE_NUMBER_TOO_NEW: u64 = 4; // transaction sequence number is too new
pub const EACCOUNT_DOES_NOT_EXIST: u64 = 5; // transaction sender's account does not exist
pub const ECANT_PAY_GAS_DEPOSIT: u64 = 6; // insufficient balance to pay for gas deposit

/// Generic error codes. These codes don't have any special meaning for the VM, but they are useful
/// conventions for debugging
pub const EINSUFFICIENT_BALANCE: u64 = 10; // withdrawing more than an account contains
pub const EINSUFFICIENT_PRIVILEGES: u64 = 11; // user lacks the credentials to do something

pub const EASSERT_ERROR: u64 = 42; // catch-all error code for assert failures

pub type VMResult<T> = ::std::result::Result<T, VMStatus>;
pub type BinaryLoaderResult<T> = ::std::result::Result<T, VMStatus>;

impl Location {
    pub fn new() -> Self {
        Location {}
    }
}

////////////////////////////////////////////////////////////////////////////
/// Conversion functions from internal VM statuses into external VM statuses
////////////////////////////////////////////////////////////////////////////

pub fn vm_status_of_result<T>(result: &VMResult<T>) -> VMStatus {
    match result {
        Ok(_) => VMStatus::new(StatusCode::EXECUTED),
        Err(err) => err.clone(),
    }
}

// FUTURE: At the moment we can't pass transaction metadata or the signed transaction due to
// restrictions in the two places that this function is called. We therefore just pass through what
// we need at the moment---the sender address---but we may want/need to pass more data later on.
pub fn convert_prologue_runtime_error(err: &VMStatus, txn_sender: &AccountAddress) -> VMStatus {
    if err.major_status == StatusCode::ABORTED {
        match err.sub_status {
            // Invalid authentication key
            Some(EBAD_ACCOUNT_AUTHENTICATION_KEY) => VMStatus::new(StatusCode::INVALID_AUTH_KEY),
            // Sequence number too old
            Some(ESEQUENCE_NUMBER_TOO_OLD) => VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_OLD),
            // Sequence number too new
            Some(ESEQUENCE_NUMBER_TOO_NEW) => VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_NEW),
            // Sequence number too new
            Some(EACCOUNT_DOES_NOT_EXIST) => {
                let error_msg = format!("sender address: {}", txn_sender);
                VMStatus::new(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST).with_message(error_msg)
            }
            // Can't pay for transaction gas deposit/fee
            Some(ECANT_PAY_GAS_DEPOSIT) => {
                VMStatus::new(StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE)
            }
            // This should never happen...
            _ => err.clone(),
        }
    } else {
        err.clone()
    }
}

pub fn vm_error(location: Location, err: StatusCode) -> VMStatus {
    let msg = format!("At location {:#?}", location);
    VMStatus::new(err).with_message(msg)
}

//pub fn bytecode_offset_err(offset: usize, len: usize, bytecode_offset: usize, kind: IndexKind,
// status: StatusCode) -> VMStatus {
pub fn bytecode_offset_err(
    kind: IndexKind,
    offset: usize,
    len: usize,
    bytecode_offset: usize,
    status: StatusCode,
) -> VMStatus {
    let msg = format!(
        "Index {} out of bounds for {} at bytecode offset {} while indexing {}",
        offset, len, bytecode_offset, kind
    );
    VMStatus::new(status).with_message(msg)
}

pub fn bounds_error(kind: IndexKind, idx: usize, len: usize, err: StatusCode) -> VMStatus {
    let msg = format!(
        "Index {} out of bounds for {} while indexing {}",
        idx, len, kind
    );
    VMStatus::new(err).with_message(msg)
}

pub fn verification_error(kind: IndexKind, idx: usize, err: StatusCode) -> VMStatus {
    let msg = format!("at index {} while indexing {}", idx, kind);
    VMStatus::new(err).with_message(msg)
}

pub fn append_err_info(status: VMStatus, kind: IndexKind, idx: usize) -> VMStatus {
    let msg = format!("at index {} while indexing {}", idx, kind);
    status.append_message_with_separator(' ', msg)
}

pub fn err_at_offset(status: StatusCode, offset: usize) -> VMStatus {
    let msg = format!("At offset {}", offset);
    VMStatus::new(status).with_message(msg)
}
