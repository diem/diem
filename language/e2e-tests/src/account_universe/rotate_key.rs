// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_universe::{AUTransactionGen, AccountUniverse},
    common_transactions::rotate_key_txn,
    gas_costs,
};
use libra_crypto::ed25519::{compat::keypair_strategy, *};
use libra_proptest_helpers::Index;
use libra_types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

/// Represents a rotate-key transaction performed in the account universe.
#[derive(Arbitrary, Clone, Debug)]
#[proptest(no_params)]
pub struct RotateKeyGen {
    sender: Index,
    #[proptest(strategy = "keypair_strategy()")]
    new_key: (Ed25519PrivateKey, Ed25519PublicKey),
}

impl AUTransactionGen for RotateKeyGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let sender = universe.pick(&self.sender).1;

        let new_key_hash = AccountAddress::from_public_key(&self.new_key.1);
        let txn = rotate_key_txn(sender.account(), new_key_hash, sender.sequence_number);

        // This should work all the time except for if the balance is too low for gas.
        let mut gas_cost = 0;
        let enough_max_gas = sender.balance >= gas_costs::TXN_RESERVED;
        let status = if enough_max_gas {
            sender.sequence_number += 1;
            gas_cost = sender.rotate_key_gas_cost();
            sender.balance -= gas_cost;
            let (privkey, pubkey) = (self.new_key.0.clone(), self.new_key.1.clone());
            sender.rotate_key(privkey, pubkey);

            TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED))
        } else {
            TransactionStatus::Discard(VMStatus::new(
                StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE,
            ))
        };

        (txn, (status, gas_cost))
    }
}
