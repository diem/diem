// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! mempool is used to track transactions which have been submitted but not yet
//! agreed upon.
use crate::{
    core_mempool::{
        index::TxnPointer,
        transaction::{MempoolTransaction, TimelineState},
        transaction_store::TransactionStore,
        ttl_cache::TtlCache,
    },
    counters,
    logging::{LogEntry, LogSchema, TxnsLog},
};
use diem_config::config::NodeConfig;
use diem_logger::prelude::*;
use diem_trace::prelude::*;
use diem_types::{
    account_address::AccountAddress,
    mempool_status::{MempoolStatus, MempoolStatusCode},
    transaction::{GovernanceRole, SignedTransaction},
};
use std::{
    cmp::max,
    collections::HashSet,
    time::{Duration, SystemTime},
};

pub struct Mempool {
    // stores metadata of all transactions in mempool (of all states)
    transactions: TransactionStore,

    sequence_number_cache: TtlCache<AccountAddress, u64>,
    // for each transaction, entry with timestamp is added when transaction enters mempool
    // used to measure e2e latency of transaction in system, as well as time it takes to pick it up
    // by consensus
    pub(crate) metrics_cache: TtlCache<(AccountAddress, u64), SystemTime>,
    pub system_transaction_timeout: Duration,
}

impl Mempool {
    pub fn new(config: &NodeConfig) -> Self {
        Mempool {
            transactions: TransactionStore::new(&config.mempool),
            sequence_number_cache: TtlCache::new(config.mempool.capacity, Duration::from_secs(100)),
            metrics_cache: TtlCache::new(config.mempool.capacity, Duration::from_secs(100)),
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
        trace_event!("mempool:remove_transaction", {"txn", sender, sequence_number});
        trace!(
            LogSchema::new(LogEntry::RemoveTxn).txns(TxnsLog::new_txn(*sender, sequence_number)),
            is_rejected = is_rejected
        );
        let metric_label = if is_rejected {
            counters::COMMIT_REJECTED_LABEL
        } else {
            counters::COMMIT_ACCEPTED_LABEL
        };
        self.log_latency(*sender, sequence_number, metric_label);
        self.metrics_cache.remove(&(*sender, sequence_number));

        let current_seq_number = self
            .sequence_number_cache
            .remove(&sender)
            .unwrap_or_default();

        if is_rejected {
            if sequence_number >= current_seq_number {
                self.transactions
                    .reject_transaction(&sender, sequence_number);
            }
        } else {
            // update current cached sequence number for account
            let new_seq_number = max(current_seq_number, sequence_number + 1);
            self.sequence_number_cache.insert(*sender, new_seq_number);
            self.transactions
                .commit_transaction(&sender, new_seq_number);
        }
    }

    fn log_latency(&mut self, account: AccountAddress, sequence_number: u64, metric: &str) {
        if let Some(&creation_time) = self.metrics_cache.get(&(account, sequence_number)) {
            if let Ok(time_delta) = SystemTime::now().duration_since(creation_time) {
                counters::CORE_MEMPOOL_TXN_COMMIT_LATENCY
                    .with_label_values(&[metric])
                    .observe(time_delta.as_secs_f64());
            }
        }
    }

    /// Used to add a transaction to the Mempool
    /// Performs basic validation: checks account's sequence number
    pub(crate) fn add_txn(
        &mut self,
        txn: SignedTransaction,
        gas_amount: u64,
        ranking_score: u64,
        db_sequence_number: u64,
        timeline_state: TimelineState,
        governance_role: GovernanceRole,
    ) -> MempoolStatus {
        trace_event!("mempool::add_txn", {"txn", txn.sender(), txn.sequence_number()});
        trace!(
            LogSchema::new(LogEntry::AddTxn)
                .txns(TxnsLog::new_txn(txn.sender(), txn.sequence_number())),
            committed_seq_number = db_sequence_number
        );
        let cached_value = self.sequence_number_cache.get(&txn.sender());
        let sequence_number =
            cached_value.map_or(db_sequence_number, |value| max(*value, db_sequence_number));
        self.sequence_number_cache
            .insert(txn.sender(), sequence_number);

        // don't accept old transactions (e.g. seq is less than account's current seq_number)
        if txn.sequence_number() < sequence_number {
            return MempoolStatus::new(MempoolStatusCode::InvalidSeqNumber).with_message(format!(
                "transaction sequence number is {}, current sequence number is  {}",
                txn.sequence_number(),
                sequence_number,
            ));
        }

        let expiration_time =
            diem_infallible::duration_since_epoch() + self.system_transaction_timeout;
        if timeline_state != TimelineState::NonQualified {
            self.metrics_cache
                .insert((txn.sender(), txn.sequence_number()), SystemTime::now());
        }

        let txn_info = MempoolTransaction::new(
            txn,
            expiration_time,
            gas_amount,
            ranking_score,
            timeline_state,
            governance_role,
        );

        self.transactions.insert(txn_info, sequence_number)
    }

    /// Fetches next block of transactions for consensus
    /// `batch_size` - size of requested block
    /// `seen_txns` - transactions that were sent to Consensus but were not committed yet
    ///  Mempool should filter out such transactions
    #[allow(clippy::explicit_counter_loop)]
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
        let seen_size = seen.len();
        let mut txn_walked = 0usize;
        // iterate over the queue of transactions based on gas price
        'main: for txn in self.transactions.iter_queue() {
            txn_walked += 1;
            if seen.contains(&TxnPointer::from(txn)) {
                continue;
            }
            let seq = txn.sequence_number;
            let account_sequence_number = self.sequence_number_cache.get(&txn.address);
            let seen_previous = seq > 0 && seen.contains(&(txn.address, seq - 1));
            // include transaction if it's "next" for given account or
            // we've already sent its ancestor to Consensus
            if seen_previous || account_sequence_number == Some(&seq) {
                let ptr = TxnPointer::from(txn);
                seen.insert(ptr);
                trace_event!("mempool::get_block", {"txn", txn.address, txn.sequence_number});
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
        let result_size = result.len();
        // convert transaction pointers to real values
        let mut block_log = TxnsLog::new();
        let block: Vec<_> = result
            .into_iter()
            .filter_map(|(address, seq)| {
                block_log.add(address, seq);
                self.transactions.get(&address, seq)
            })
            .collect();

        debug!(
            LogSchema::new(LogEntry::GetBlock).txns(block_log),
            seen_consensus = seen_size,
            walked = txn_walked,
            seen_after = seen.len(),
            result_size = result_size,
            block_size = block.len()
        );
        for transaction in &block {
            self.log_latency(
                transaction.sender(),
                transaction.sequence_number(),
                counters::GET_BLOCK_STAGE_LABEL,
            );
        }
        block
    }

    /// periodic core mempool garbage collection
    /// removes all expired transactions
    /// clears expired entries in metrics cache and sequence number cache
    pub(crate) fn gc(&mut self) {
        let now = SystemTime::now();
        self.transactions.gc_by_system_ttl(&self.metrics_cache);
        self.metrics_cache.gc(now);
        self.sequence_number_cache.gc(now);
    }

    /// Garbage collection based on client-specified expiration time
    pub(crate) fn gc_by_expiration_time(&mut self, block_time: Duration) {
        self.transactions
            .gc_by_expiration_time(block_time, &self.metrics_cache);
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

    /// Read transactions from timeline from `start_id` (exclusive) to `end_id` (inclusive)
    pub(crate) fn timeline_range(&mut self, start_id: u64, end_id: u64) -> Vec<SignedTransaction> {
        self.transactions.timeline_range(start_id, end_id)
    }

    pub fn gen_snapshot(&self) -> TxnsLog {
        self.transactions.gen_snapshot(&self.metrics_cache)
    }

    #[cfg(test)]
    pub fn get_parking_lot_size(&self) -> usize {
        self.transactions.get_parking_lot_size()
    }
}
