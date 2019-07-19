// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_universe::{AUTransactionGen, AccountUniverse},
    common_transactions::rotate_key_txn,
    gas_costs,
};
use crypto::{utils::keypair_strategy, PrivateKey, PublicKey};
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use proptest_helpers::Index;
use types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus, VMValidationStatus},
};

/// Represents a rotate-key transaction performed in the account universe.
#[derive(Arbitrary, Clone, Debug)]
#[proptest(no_params)]
pub struct RotateKeyGen {
    sender: Index,
    #[proptest(strategy = "keypair_strategy()")]
    new_key: (PrivateKey, PublicKey),
}

impl AUTransactionGen for RotateKeyGen {
    fn apply(&self, universe: &mut AccountUniverse) -> (SignedTransaction, TransactionStatus) {
        let sender_idx = self.sender.index(universe.num_accounts());
        let mut sender = &mut universe.accounts[sender_idx];

        let new_key_hash = AccountAddress::from(self.new_key.1);
        let txn = rotate_key_txn(sender.account(), new_key_hash, sender.sequence_number);

        // This should work all the time except for if the balance is too low for gas.
        let enough_max_gas = sender.balance >= gas_costs::TXN_RESERVED;
        let status = if enough_max_gas {
            sender.sequence_number += 1;
            sender.balance -= *gas_costs::ROTATE_KEY;
            let (privkey, pubkey) = (self.new_key.0.clone(), self.new_key.1);
            sender.rotate_key(privkey, pubkey);

            TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed))
        } else {
            TransactionStatus::Discard(VMStatus::Validation(
                VMValidationStatus::InsufficientBalanceForTransactionFee,
            ))
        };

        (txn, status)
    }
}
