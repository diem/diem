// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vm_validator::TransactionValidation;
use anyhow::Result;
use libra_state_view::StateView;
use libra_types::{
    account_address::AccountAddress,
    on_chain_config::OnChainConfigPayload,
    transaction::{SignedTransaction, VMValidatorResult},
    vm_error::{StatusCode, VMStatus},
};
use libra_vm::VMValidator;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct MockVMValidator;

impl VMValidator for MockVMValidator {
    fn validate_transaction(
        &self,
        _transaction: SignedTransaction,
        _state_view: &dyn StateView,
    ) -> VMValidatorResult {
        VMValidatorResult::new(None, 0, false)
    }
}

impl TransactionValidation for MockVMValidator {
    type ValidationInstance = MockVMValidator;
    fn validate_transaction(&self, txn: SignedTransaction) -> Result<VMValidatorResult> {
        let txn = match txn.check_signature() {
            Ok(txn) => txn,
            Err(_) => {
                return Ok(VMValidatorResult::new(
                    Some(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
                    0,
                    false,
                ))
            }
        };

        let sender = txn.sender();
        let account_dne_test_add =
            AccountAddress::try_from(&[0 as u8; AccountAddress::LENGTH]).unwrap();
        let invalid_sig_test_add =
            AccountAddress::try_from(&[1 as u8; AccountAddress::LENGTH]).unwrap();
        let insufficient_balance_test_add =
            AccountAddress::try_from(&[2 as u8; AccountAddress::LENGTH]).unwrap();
        let seq_number_too_new_test_add =
            AccountAddress::try_from(&[3 as u8; AccountAddress::LENGTH]).unwrap();
        let seq_number_too_old_test_add =
            AccountAddress::try_from(&[4 as u8; AccountAddress::LENGTH]).unwrap();
        let txn_expiration_time_test_add =
            AccountAddress::try_from(&[5 as u8; AccountAddress::LENGTH]).unwrap();
        let invalid_auth_key_test_add =
            AccountAddress::try_from(&[6 as u8; AccountAddress::LENGTH]).unwrap();
        let ret = if sender == account_dne_test_add {
            Some(VMStatus::new(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST))
        } else if sender == invalid_sig_test_add {
            Some(VMStatus::new(StatusCode::INVALID_SIGNATURE))
        } else if sender == insufficient_balance_test_add {
            Some(VMStatus::new(
                StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE,
            ))
        } else if sender == seq_number_too_new_test_add {
            Some(VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_NEW))
        } else if sender == seq_number_too_old_test_add {
            Some(VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_OLD))
        } else if sender == txn_expiration_time_test_add {
            Some(VMStatus::new(StatusCode::TRANSACTION_EXPIRED))
        } else if sender == invalid_auth_key_test_add {
            Some(VMStatus::new(StatusCode::INVALID_AUTH_KEY))
        } else {
            None
        };
        Ok(VMValidatorResult::new(ret, 0, false))
    }

    fn restart(&mut self, _config: OnChainConfigPayload) -> Result<()> {
        unimplemented!();
    }
}
