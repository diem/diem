// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_validator::TransactionValidation;
use futures::future::{ok, Future};
use state_view::StateView;
use std::convert::TryFrom;
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    transaction::SignedTransaction,
    vm_error::{VMStatus, VMValidationStatus},
};
use vm_runtime::VMVerifier;

#[derive(Clone)]
pub struct MockVMValidator;

impl VMVerifier for MockVMValidator {
    fn validate_transaction(
        &self,
        _transaction: SignedTransaction,
        _state_view: &dyn StateView,
    ) -> Option<VMStatus> {
        None
    }
}

impl TransactionValidation for MockVMValidator {
    type ValidationInstance = MockVMValidator;
    fn validate_transaction(
        &self,
        txn: SignedTransaction,
    ) -> Box<dyn Future<Item = Option<VMStatus>, Error = failure::Error> + Send> {
        let txn = match txn.check_signature() {
            Ok(txn) => txn,
            Err(_) => {
                return Box::new(ok(Some(VMStatus::Validation(
                    VMValidationStatus::InvalidSignature,
                ))))
            }
        };

        let sender = txn.sender();
        let account_dne_test_add = AccountAddress::try_from(&[0 as u8; ADDRESS_LENGTH]).unwrap();
        let invalid_sig_test_add = AccountAddress::try_from(&[1 as u8; ADDRESS_LENGTH]).unwrap();
        let insufficient_balance_test_add =
            AccountAddress::try_from(&[2 as u8; ADDRESS_LENGTH]).unwrap();
        let seq_number_too_new_test_add =
            AccountAddress::try_from(&[3 as u8; ADDRESS_LENGTH]).unwrap();
        let seq_number_too_old_test_add =
            AccountAddress::try_from(&[4 as u8; ADDRESS_LENGTH]).unwrap();
        let txn_expiration_time_test_add =
            AccountAddress::try_from(&[5 as u8; ADDRESS_LENGTH]).unwrap();
        let invalid_auth_key_test_add =
            AccountAddress::try_from(&[6 as u8; ADDRESS_LENGTH]).unwrap();
        let ret = if sender == account_dne_test_add {
            Some(VMStatus::Validation(
                VMValidationStatus::SendingAccountDoesNotExist("TEST".to_string()),
            ))
        } else if sender == invalid_sig_test_add {
            Some(VMStatus::Validation(VMValidationStatus::InvalidSignature))
        } else if sender == insufficient_balance_test_add {
            Some(VMStatus::Validation(
                VMValidationStatus::InsufficientBalanceForTransactionFee,
            ))
        } else if sender == seq_number_too_new_test_add {
            Some(VMStatus::Validation(
                VMValidationStatus::SequenceNumberTooNew,
            ))
        } else if sender == seq_number_too_old_test_add {
            Some(VMStatus::Validation(
                VMValidationStatus::SequenceNumberTooOld,
            ))
        } else if sender == txn_expiration_time_test_add {
            Some(VMStatus::Validation(VMValidationStatus::TransactionExpired))
        } else if sender == invalid_auth_key_test_add {
            Some(VMStatus::Validation(VMValidationStatus::InvalidAuthKey))
        } else {
            None
        };
        Box::new(ok(ret))
    }
}
