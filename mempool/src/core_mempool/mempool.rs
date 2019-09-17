// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! mempool is used to track transactions which have been submitted but not yet
//! agreed upon.
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    core_mempool::{
        index::TxnPointer,
        transaction::{MempoolAddTransactionStatus, MempoolTransaction, TimelineState},
        transaction_store::TransactionStore,
    },
    proto::shared::mempool_status::MempoolAddTransactionStatusCode,
    OP_COUNTERS,
};
use chrono::Utc;
use config::config::NodeConfig;
use logger::prelude::*;
use lru_cache::LruCache;
use std::{
    cmp::{max, min},
    collections::HashSet,
    convert::TryFrom,
};
use ttl_cache::TtlCache;
use types::{account_address::AccountAddress, transaction::SignedTransaction};
/// TODO doc
pub struct Mempool {
    // stores metadata of all transactions in mempool (of all states)
    transactions: TransactionStore,

    sequence_number_cache: LruCache<AccountAddress, u64>,
    // temporary DS. TODO: eventually retire it
    // for each transaction, entry with timestamp is added when transaction enters mempool
    // used to measure e2e latency of transaction in system, as well as time it takes to pick it up
    // by consensus
    pub(crate) metrics_cache: TtlCache<(AccountAddress, u64), i64>,
    /// TODO doc
    pub system_transaction_timeout: Duration,
}

impl Mempool {
    pub(crate) fn new(config: &NodeConfig) -> Self {
        Mempool {
            transactions: TransactionStore::new(&config.mempool),
            sequence_number_cache: LruCache::new(config.mempool.sequence_cache_capacity),
            metrics_cache: TtlCache::new(config.mempool.capacity),
            system_transaction_timeout: Duration::from_secs(
                config.mempool.system_transaction_timeout_secs,
            ),
        }
    }

    /// This function will be called once the transaction has been stored
    pub(crate) fn remove_transaction(
        &mut self,
        sender: &AccountAddress,
        sequence_number: u64,
        is_rejected: bool,
    ) {
        debug!(
            "[Mempool] Removing transaction from mempool: {}:{}",
            sender, sequence_number
        );
        self.log_latency(sender.clone(), sequence_number, "e2e.latency");
        self.metrics_cache.remove(&(*sender, sequence_number));

        // update current cached sequence number for account
        let cached_value = self
            .sequence_number_cache
            .remove(sender)
            .unwrap_or_default();

        let new_sequence_number = if is_rejected {
            min(sequence_number, cached_value)
        } else {
            max(cached_value, sequence_number + 1)
        };
        self.sequence_number_cache
            .insert(sender.clone(), new_sequence_number);

        self.transactions
            .commit_transaction(&sender, sequence_number);
    }

    fn log_latency(&mut self, account: AccountAddress, sequence_number: u64, metric: &str) {
        if let Some(&creation_time) = self.metrics_cache.get(&(account, sequence_number)) {
            if let Ok(time_delta_ms) = u64::try_from(Utc::now().timestamp_millis() - creation_time)
            {
                OP_COUNTERS.observe_duration(metric, Duration::from_millis(time_delta_ms));
            }
        }
    }

    fn get_required_balance(&mut self, txn: &SignedTransaction, gas_amount: u64) -> u64 {
        txn.gas_unit_price() * gas_amount + self.transactions.get_required_balance(&txn.sender())
    }

    /// Used to add a transaction to the Mempool
    /// Performs basic validation: checks account's balance and sequence number
    pub(crate) fn add_txn(
        &mut self,
        txn: SignedTransaction,
        gas_amount: u64,
        db_sequence_number: u64,
        balance: u64,
        timeline_state: TimelineState,
    ) -> MempoolAddTransactionStatus {
        debug!(
            "[Mempool] Adding transaction to mempool: {}:{}",
            &txn.sender(),
            db_sequence_number
        );

        let required_balance = self.get_required_balance(&txn, gas_amount);
        if balance < required_balance {
            return MempoolAddTransactionStatus::new(
                MempoolAddTransactionStatusCode::InsufficientBalance,
                format!(
                    "balance: {}, required_balance: {}, gas_amount: {}",
                    balance, required_balance, gas_amount
                ),
            );
        }

        let cached_value = self.sequence_number_cache.get_mut(&txn.sender());
        let sequence_number = match cached_value {
            Some(value) => max(*value, db_sequence_number),
            None => db_sequence_number,
        };
        self.sequence_number_cache
            .insert(txn.sender(), sequence_number);

        // don't accept old transactions (e.g. seq is less than account's current seq_number)
        if txn.sequence_number() < sequence_number {
            return MempoolAddTransactionStatus::new(
                MempoolAddTransactionStatusCode::InvalidSeqNumber,
                format!(
                    "transaction sequence number is {}, current sequence number is  {}",
                    txn.sequence_number(),
                    sequence_number,
                ),
            );
        }

        let expiration_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("init timestamp failure")
            + self.system_transaction_timeout;
        if timeline_state != TimelineState::NonQualified {
            self.metrics_cache.insert(
                (txn.sender(), txn.sequence_number()),
                Utc::now().timestamp_millis(),
                Duration::from_secs(100),
            );
        }

        let txn_info = MempoolTransaction::new(txn, expiration_time, gas_amount, timeline_state);

        let status = self.transactions.insert(txn_info, sequence_number);
        OP_COUNTERS.inc(&format!("insert.{:?}", status));
        status
    }

    /// Fetches next block of transactions for consensus
    /// `batch_size` - size of requested block
    /// `seen_txns` - transactions that were sent to Consensus but were not committed yet
    ///  Mempool should filter out such transactions
    pub(crate) fn get_block(
        &mut self,
        batch_size: u64,
        mut seen: HashSet<TxnPointer>,
    ) -> Vec<SignedTransaction> {
        let mut result = vec![];
        // Helper DS. Helps to mitigate scenarios where account submits several transactions
        // with increasing gas price (e.g. user submits transactions with sequence number 1, 2
        // and gas_price 1, 10 respectively)
        // Later txn has higher gas price and will be observed first in priority index iterator,
        // but can't be executed before first txn. Once observed, such txn will be saved in
        // `skipped` DS and rechecked once it's ancestor becomes available
        let mut skipped = HashSet::new();

        // iterate over the queue of transactions based on gas price
        'main: for txn in self.transactions.iter_queue() {
            if seen.contains(&TxnPointer::from(txn)) {
                continue;
            }
            let mut seq = txn.sequence_number;
            let account_sequence_number = self.sequence_number_cache.get_mut(&txn.address);
            let seen_previous = seq > 0 && seen.contains(&(txn.address, seq - 1));
            // include transaction if it's "next" for given account or
            // we've already sent its ancestor to Consensus
            if seen_previous || account_sequence_number == Some(&mut seq) {
                let ptr = TxnPointer::from(txn);
                seen.insert(ptr);
                result.push(ptr);
                if (result.len() as u64) == batch_size {
                    break;
                }

                // check if we can now include some transactions
                // that were skipped before for given account
                let mut skipped_txn = (txn.address, seq + 1);
                while skipped.contains(&skipped_txn) {
                    seen.insert(skipped_txn);
                    result.push(skipped_txn);
                    if (result.len() as u64) == batch_size {
                        break 'main;
                    }
                    skipped_txn = (txn.address, skipped_txn.1 + 1);
                }
            } else {
                skipped.insert(TxnPointer::from(txn));
            }
        }
        // convert transaction pointers to real values
        let block: Vec<_> = result
            .into_iter()
            .filter_map(|(address, seq)| self.transactions.get(&address, seq))
            .collect();
        for transaction in &block {
            self.log_latency(
                transaction.sender(),
                transaction.sequence_number(),
                "txn_pre_consensus_s",
            );
        }
        block
    }

    /// TTL based garbage collection. Remove all transactions that got expired
    pub(crate) fn gc_by_system_ttl(&mut self) {
        self.transactions.gc_by_system_ttl();
    }

    /// Garbage collection based on client-specified expiration time
    pub(crate) fn gc_by_expiration_time(&mut self, block_time: Duration) {
        self.transactions.gc_by_expiration_time(block_time);
    }

    /// Read `count` transactions from timeline since `timeline_id`
    /// Returns block of transactions and new last_timeline_id
    pub(crate) fn read_timeline(
        &mut self,
        timeline_id: u64,
        count: usize,
    ) -> (Vec<SignedTransaction>, u64) {
        self.transactions.read_timeline(timeline_id, count)
    }

    /// Check the health of core mempool.
    pub(crate) fn health_check(&self) -> bool {
        self.transactions.health_check()
    }
}
