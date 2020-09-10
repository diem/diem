// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_universe::{AUTransactionGen, AccountUniverse},
    common_transactions::rotate_key_txn,
    gas_costs,
};
use libra_crypto::{
    ed25519::{self, Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use libra_proptest_helpers::Index;
use libra_types::{
    transaction::{authenticator::AuthenticationKey, SignedTransaction, TransactionStatus},
    vm_status::{KeptVMStatus, StatusCode},
};
use proptest::prelude::*;
use proptest_derive::Arbitrary;

/// Represents a rotate-key transaction performed in the account universe.
#[derive(Arbitrary, Clone, Debug)]
#[proptest(no_params)]
pub struct RotateKeyGen {
    sender: Index,
    #[proptest(strategy = "ed25519::keypair_strategy()")]
    new_keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
}

impl AUTransactionGen for RotateKeyGen {
    fn apply(
        &self,
        universe: &mut AccountUniverse,
    ) -> (SignedTransaction, (TransactionStatus, u64)) {
        let sender = universe.pick(self.sender).1;

        let key_hash = AuthenticationKey::ed25519(&self.new_keypair.public_key).to_vec();
        let txn = rotate_key_txn(sender.account(), key_hash, sender.sequence_number);

        // This should work all the time except for if the balance is too low for gas.
        let mut gas_used = 0;
        let enough_max_gas = sender.balance >= gas_costs::TXN_RESERVED * txn.gas_unit_price();
        let status = if enough_max_gas {
            sender.sequence_number += 1;
            gas_used = sender.rotate_key_gas_cost();
            sender.balance -= gas_used * txn.gas_unit_price();
            sender.rotate_key(
                self.new_keypair.private_key.clone(),
                self.new_keypair.public_key.clone(),
            );

            TransactionStatus::Keep(KeptVMStatus::Executed)
        } else {
            TransactionStatus::Discard(StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE)
        };

        (txn, (status, gas_used))
    }
}
