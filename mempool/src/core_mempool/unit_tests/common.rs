// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::core_mempool::{CoreMempool, TimelineState, TxnPointer};
use anyhow::{format_err, Result};
use libra_config::config::NodeConfig;
use libra_crypto::{ed25519::Ed25519PrivateKey, TPrivateKey, Uniform};
use libra_types::{
    account_address::AccountAddress,
    account_config::LBR_NAME,
    mempool_status::MempoolStatusCode,
    transaction::{RawTransaction, Script, SignedTransaction},
};
use once_cell::sync::Lazy;
use rand::{rngs::StdRng, SeedableRng};
use std::{collections::HashSet, iter::FromIterator};

pub(crate) fn setup_mempool() -> (CoreMempool, ConsensusMock) {
    (
        CoreMempool::new(&NodeConfig::random()),
        ConsensusMock::new(),
    )
}

static ACCOUNTS: Lazy<Vec<AccountAddress>> =
    Lazy::new(|| vec![AccountAddress::random(), AccountAddress::random()]);

#[derive(Clone)]
pub struct TestTransaction {
    pub(crate) address: usize,
    pub(crate) sequence_number: u64,
    gas_price: u64,
}

impl TestTransaction {
    pub(crate) fn new(address: usize, sequence_number: u64, gas_price: u64) -> Self {
        Self {
            address,
            sequence_number,
            gas_price,
        }
    }

    pub(crate) fn make_signed_transaction_with_expiration_time(
        &self,
        exp_time: std::time::Duration,
    ) -> SignedTransaction {
        self.make_signed_transaction_impl(100, exp_time)
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

    pub(crate) fn make_signed_transaction(&self) -> SignedTransaction {
        self.make_signed_transaction_impl(100, std::time::Duration::from_secs(u64::max_value()))
    }

    fn make_signed_transaction_impl(
        &self,
        max_gas_amount: u64,
        exp_time: std::time::Duration,
    ) -> SignedTransaction {
        let raw_txn = RawTransaction::new_script(
            TestTransaction::get_address(self.address),
            self.sequence_number,
            Script::new(vec![], vec![], vec![]),
            max_gas_amount,
            self.gas_price,
            LBR_NAME.to_string(),
            exp_time,
        );
        let mut seed: [u8; 32] = [0u8; 32];
        seed[..4].copy_from_slice(&[1, 2, 3, 4]);
        let mut rng: StdRng = StdRng::from_seed(seed);
        let privkey = Ed25519PrivateKey::generate(&mut rng);
        raw_txn
            .sign(&privkey, privkey.public_key())
            .expect("Failed to sign raw transaction.")
            .into_inner()
    }

    pub(crate) fn get_address(address: usize) -> AccountAddress {
        ACCOUNTS[address]
    }
}

// adds transactions to mempool
pub(crate) fn add_txns_to_mempool(
    pool: &mut CoreMempool,
    txns: Vec<TestTransaction>,
) -> Vec<SignedTransaction> {
    let mut transactions = vec![];
    for transaction in txns {
        let txn = transaction.make_signed_transaction();
        pool.add_txn(
            txn.clone(),
            0,
            txn.gas_unit_price(),
            0,
            TimelineState::NotReady,
        );
        transactions.push(txn);
    }
    transactions
}

pub(crate) fn add_txn(pool: &mut CoreMempool, transaction: TestTransaction) -> Result<()> {
    add_signed_txn(pool, transaction.make_signed_transaction())
}

pub(crate) fn add_signed_txn(pool: &mut CoreMempool, transaction: SignedTransaction) -> Result<()> {
    match pool
        .add_txn(
            transaction.clone(),
            0,
            transaction.gas_unit_price(),
            0,
            TimelineState::NotReady,
        )
        .code
    {
        MempoolStatusCode::Accepted => Ok(()),
        _ => Err(format_err!("insertion failure")),
    }
}

pub(crate) fn batch_add_signed_txn(
    pool: &mut CoreMempool,
    transactions: Vec<SignedTransaction>,
) -> Result<()> {
    for txn in transactions.into_iter() {
        if let Err(e) = add_signed_txn(pool, txn) {
            return Err(e);
        }
    }
    Ok(())
}

// helper struct that keeps state between `.get_block` calls. Imitates work of Consensus
pub struct ConsensusMock(HashSet<TxnPointer>);

impl ConsensusMock {
    pub(crate) fn new() -> Self {
        Self(HashSet::new())
    }

    pub(crate) fn get_block(
        &mut self,
        mempool: &mut CoreMempool,
        block_size: u64,
    ) -> Vec<SignedTransaction> {
        let block = mempool.get_block(block_size, self.0.clone());
        self.0 = self
            .0
            .union(&HashSet::from_iter(
                block.iter().map(|t| (t.sender(), t.sequence_number())),
            ))
            .cloned()
            .collect();
        block
    }
}

pub(crate) fn exist_in_metrics_cache(mempool: &CoreMempool, txn: &SignedTransaction) -> bool {
    mempool
        .metrics_cache
        .get(&(txn.sender(), txn.sequence_number()))
        .is_some()
}
