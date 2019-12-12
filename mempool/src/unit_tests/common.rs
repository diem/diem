// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;
use libra_crypto::ed25519::*;
use libra_types::{
    account_address::AccountAddress,
    transaction::{RawTransaction, Script, SignedTransaction},
};
use rand::{rngs::StdRng, SeedableRng};

lazy_static! {
    static ref ACCOUNTS: Vec<AccountAddress> =
        vec![AccountAddress::random(), AccountAddress::random()];
}

#[derive(Clone)]
pub struct TestTransaction {
    pub(crate) address: usize,
    pub(crate) sequence_number: u64,
    gas_price: u64,
}

impl TestTransaction {
    pub fn new(address: usize, sequence_number: u64, gas_price: u64) -> Self {
        Self {
            address,
            sequence_number,
            gas_price,
        }
    }

    pub(crate) fn make_signed_transaction_with_max_gas_amount(
        &self,
        max_gas_amount: u64,
    ) -> SignedTransaction {
        self.make_signed_transaction_impl(
            max_gas_amount,
            std::time::Duration::from_secs(u64::max_value()),
        )
    }

    fn make_signed_transaction_impl(
        &self,
        max_gas_amount: u64,
        exp_time: std::time::Duration,
    ) -> SignedTransaction {
        let raw_txn = RawTransaction::new_script(
            TestTransaction::get_address(self.address),
            self.sequence_number,
            Script::new(vec![], vec![]),
            max_gas_amount,
            self.gas_price,
            exp_time,
        );
        let mut seed: [u8; 32] = [0u8; 32];
        seed[..4].copy_from_slice(&[1, 2, 3, 4]);
        let mut rng: StdRng = StdRng::from_seed(seed);
        let (privkey, pubkey) = compat::generate_keypair(&mut rng);
        raw_txn
            .sign(&privkey, pubkey)
            .expect("Failed to sign raw transaction.")
            .into_inner()
    }

    pub(crate) fn get_address(address: usize) -> AccountAddress {
        ACCOUNTS[address]
    }
}
