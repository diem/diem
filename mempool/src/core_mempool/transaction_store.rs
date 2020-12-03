// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{
        index::{
            AccountTransactions, ParkingLotIndex, PriorityIndex, PriorityQueueIter, TTLIndex,
            TimelineIndex,
        },
        transaction::{MempoolTransaction, TimelineState},
        ttl_cache::TtlCache,
    },
    counters,
    logging::{LogEntry, LogEvent, LogSchema, TxnsLog},
};
use anyhow::{format_err, Result};
use diem_config::config::MempoolConfig;
use diem_logger::prelude::*;
use diem_types::{
    account_address::AccountAddress,
    mempool_status::{MempoolStatus, MempoolStatusCode},
    transaction::SignedTransaction,
};
use std::{
    collections::HashMap,
    ops::Bound,
    time::{Duration, SystemTime},
};

/// TransactionStore is in-memory storage for all transactions in mempool
pub struct TransactionStore {
    // main DS
    transactions: HashMap<AccountAddress, AccountTransactions>,

    // indexes
    priority_index: PriorityIndex,
    // TTLIndex based on client-specified expiration time
    expiration_time_index: TTLIndex,
    // TTLIndex based on system expiration time
    // we keep it separate from `expiration_time_index` so Mempool can't be clogged
    //  by old transactions even if it hasn't received commit callbacks for a while
    system_ttl_index: TTLIndex,
    timeline_index: TimelineIndex,
    // keeps track of "non-ready" txns (transactions that can't be included in next block)
    parking_lot_index: ParkingLotIndex,

    // configuration
    capacity: usize,
    capacity_per_user: usize,
}

impl TransactionStore {
    pub(crate) fn new(config: &MempoolConfig) -> Self {
        Self {
            // main DS
            transactions: HashMap::new(),

            // various indexes
            system_ttl_index: TTLIndex::new(Box::new(|t: &MempoolTransaction| t.expiration_time)),
            expiration_time_index: TTLIndex::new(Box::new(|t: &MempoolTransaction| {
                Duration::from_secs(t.txn.expiration_timestamp_secs())
            })),
            priority_index: PriorityIndex::new(),
            timeline_index: TimelineIndex::new(),
            parking_lot_index: ParkingLotIndex::new(),

            // configuration
            capacity: config.capacity,
            capacity_per_user: config.capacity_per_user,
        }
    }

    /// fetch transaction by account address + sequence_number
    pub(crate) fn get(
        &self,
        address: &AccountAddress,
        sequence_number: u64,
    ) -> Option<SignedTransaction> {
        if let Some(txn) = self
            .transactions
            .get(&address)
            .and_then(|txns| txns.get(&sequence_number))
        {
            return Some(txn.txn.clone());
        }
        None
    }

    /// insert transaction into TransactionStore
    /// performs validation checks and updates indexes
    pub(crate) fn insert(
        &mut self,
        txn: MempoolTransaction,
        current_sequence_number: u64,
    ) -> MempoolStatus {
        if self.handle_gas_price_update(&txn).is_err() {
            return MempoolStatus::new(MempoolStatusCode::InvalidUpdate).with_message(format!(
                "Failed to update gas price to {}",
                txn.get_gas_price()
            ));
        }

        if self.check_is_full_after_eviction(&txn, current_sequence_number) {
            return MempoolStatus::new(MempoolStatusCode::MempoolIsFull).with_message(format!(
                "mempool size: {}, capacity: {}",
                self.system_ttl_index.size(),
                self.capacity,
            ));
        }

        let address = txn.get_sender();
        let sequence_number = txn.get_sequence_number();

        self.transactions
            .entry(address)
            .or_insert_with(AccountTransactions::new);

        self.clean_committed_transactions(&address, current_sequence_number);

        if let Some(txns) = self.transactions.get_mut(&address) {
            // capacity check
            if txns.len() >= self.capacity_per_user {
                return MempoolStatus::new(MempoolStatusCode::TooManyTransactions).with_message(
                    format!(
                        "txns length: {} capacity per user: {}",
                        txns.len(),
                        self.capacity_per_user,
                    ),
                );
            }

            // insert into storage and other indexes
            self.system_ttl_index.insert(&txn);
            self.expiration_time_index.insert(&txn);
            txns.insert(sequence_number, txn);
            self.track_indices();
        }
        self.process_ready_transactions(&address, current_sequence_number);
        MempoolStatus::new(MempoolStatusCode::Accepted)
    }

    fn track_indices(&self) {
        counters::CORE_MEMPOOL_INDEX_SIZE
            .with_label_values(&[counters::SYSTEM_TTL_INDEX_LABEL])
            .set(self.system_ttl_index.size() as i64);
        counters::CORE_MEMPOOL_INDEX_SIZE
            .with_label_values(&[counters::EXPIRATION_TIME_INDEX_LABEL])
            .set(self.expiration_time_index.size() as i64);
        counters::CORE_MEMPOOL_INDEX_SIZE
            .with_label_values(&[counters::PRIORITY_INDEX_LABEL])
            .set(self.priority_index.size() as i64);
        counters::CORE_MEMPOOL_INDEX_SIZE
            .with_label_values(&[counters::PARKING_LOT_INDEX_LABEL])
            .set(self.parking_lot_index.size() as i64);
        counters::CORE_MEMPOOL_INDEX_SIZE
            .with_label_values(&[counters::TIMELINE_INDEX_LABEL])
            .set(self.timeline_index.size() as i64);
    }

    /// checks if Mempool is full
    /// If it's full, tries to free some space by evicting transactions from ParkingLot
    /// We only evict on attempt to insert a transaction that would be ready for broadcast upon insertion
    fn check_is_full_after_eviction(
        &mut self,
        txn: &MempoolTransaction,
        curr_sequence_number: u64,
    ) -> bool {
        if self.system_ttl_index.size() >= self.capacity
            && self.check_txn_ready(txn, curr_sequence_number)
        {
            // try to free some space in Mempool from ParkingLot by evicting a non-ready txn
            if let Some((address, sequence_number)) = self.parking_lot_index.get_poppable() {
                if let Some(txn) = self
                    .transactions
                    .get_mut(&address)
                    .and_then(|txns| txns.remove(&sequence_number))
                {
                    debug!(
                        LogSchema::new(LogEntry::MempoolFullEvictedTxn).txns(TxnsLog::new_txn(
                            txn.get_sender(),
                            txn.get_sequence_number()
                        ))
                    );
                    self.index_remove(&txn);
                }
            }
        }
        self.system_ttl_index.size() >= self.capacity
    }

    /// check if a transaction would be ready for broadcast in mempool upon insertion (without inserting it)
    /// Two ways this can happen:
    /// 1. txn sequence number == curr_sequence_number
    /// (this handles both cases where (1) txn is first possible txn for an account
    /// and (2) previous txn is committed)
    /// 2. the txn before this is ready for broadcast but not yet committed
    fn check_txn_ready(&mut self, txn: &MempoolTransaction, curr_sequence_number: u64) -> bool {
        let tx_sequence_number = txn.get_sequence_number();
        if tx_sequence_number == curr_sequence_number {
            return true;
        } else if tx_sequence_number == 0 {
            // shouldn't really get here because filtering out old txn sequence numbers happens earlier in workflow
            unreachable!("[mempool] already committed txn detected, cannot be checked for readiness upon insertion");
        }

        // check previous txn in sequence is ready
        if let Some(account_txns) = self.transactions.get(&txn.get_sender()) {
            if let Some(prev_txn) = account_txns.get(&(tx_sequence_number - 1)) {
                if let TimelineState::Ready(_) = prev_txn.timeline_state {
                    return true;
                }
            }
        }
        false
    }

    /// check if transaction is already present in Mempool
    /// e.g. given request is update
    /// we allow increase in gas price to speed up process
    fn handle_gas_price_update(&mut self, txn: &MempoolTransaction) -> Result<()> {
        if let Some(txns) = self.transactions.get_mut(&txn.get_sender()) {
            if let Some(current_version) = txns.get_mut(&txn.get_sequence_number()) {
                if current_version.txn.max_gas_amount() == txn.txn.max_gas_amount()
                    && current_version.txn.payload() == txn.txn.payload()
                    && current_version.txn.expiration_timestamp_secs()
                        == txn.txn.expiration_timestamp_secs()
                    && current_version.get_gas_price() < txn.get_gas_price()
                {
                    if let Some(txn) = txns.remove(&txn.get_sequence_number()) {
                        self.index_remove(&txn);
                    }
                } else {
                    return Err(format_err!("Invalid gas price update. txn gas price: {}, current_version gas price: {}",
                            txn.get_gas_price(),
                            current_version.get_gas_price()));
                }
            }
        }
        Ok(())
    }

    /// fixes following invariants:
    /// all transactions of given account that are sequential to current sequence number
    /// supposed to be included in both PriorityIndex (ordering for Consensus) and
    /// TimelineIndex (txns for SharedMempool)
    /// Other txns are considered to be "non-ready" and should be added to ParkingLotIndex
    fn process_ready_transactions(
        &mut self,
        address: &AccountAddress,
        current_sequence_number: u64,
    ) {
        if let Some(txns) = self.transactions.get_mut(&address) {
            let mut sequence_number = current_sequence_number;
            while let Some(txn) = txns.get_mut(&sequence_number) {
                self.priority_index.insert(txn);

                if txn.timeline_state == TimelineState::NotReady {
                    self.timeline_index.insert(txn);
                }

                // remove txn from parking lot after it has been promoted to priority_index / timeline_index,
                // i.e. txn status is ready
                self.parking_lot_index.remove(txn);
                sequence_number += 1;
            }

            let mut parking_lot_txns = 0;
            for (_, txn) in txns.range_mut((Bound::Excluded(sequence_number), Bound::Unbounded)) {
                match txn.timeline_state {
                    TimelineState::Ready(_) => {}
                    _ => {
                        self.parking_lot_index.insert(&txn);
                        parking_lot_txns += 1;
                    }
                }
            }
            trace!(
                LogSchema::new(LogEntry::ProcessReadyTxns).account(*address),
                first_ready_seq_num = current_sequence_number,
                last_ready_seq_num = sequence_number,
                num_parked_txns = parking_lot_txns,
            );
            self.track_indices();
        }
    }

    fn clean_committed_transactions(&mut self, address: &AccountAddress, sequence_number: u64) {
        // remove all previous seq number transactions for this account
        // This can happen if transactions are sent to multiple nodes and one of
        // nodes has sent the transaction to consensus but this node still has the
        // transaction sitting in mempool
        if let Some(txns) = self.transactions.get_mut(&address) {
            let mut active = txns.split_off(&sequence_number);
            let txns_for_removal = txns.clone();
            txns.clear();
            txns.append(&mut active);

            let mut rm_txns = TxnsLog::new();
            for transaction in txns_for_removal.values() {
                rm_txns.add(transaction.get_sender(), transaction.get_sequence_number());
                self.index_remove(transaction);
            }
            trace!(
                LogSchema::new(LogEntry::CleanCommittedTxn).txns(rm_txns),
                "txns cleaned with committing tx {}:{}",
                address,
                sequence_number
            );
        }
    }

    /// handles transaction commit
    /// it includes deletion of all transactions with sequence number <= `account_sequence_number`
    /// and potential promotion of sequential txns to PriorityIndex/TimelineIndex
    pub(crate) fn commit_transaction(
        &mut self,
        account: &AccountAddress,
        account_sequence_number: u64,
    ) {
        self.clean_committed_transactions(account, account_sequence_number);
        self.process_ready_transactions(account, account_sequence_number);
    }

    pub(crate) fn reject_transaction(&mut self, account: &AccountAddress, _sequence_number: u64) {
        if let Some(txns) = self.transactions.remove(&account) {
            let mut txns_log = TxnsLog::new();
            for transaction in txns.values() {
                txns_log.add(transaction.get_sender(), transaction.get_sequence_number());
                self.index_remove(&transaction);
            }
            debug!(LogSchema::new(LogEntry::CleanRejectedTxn).txns(txns_log));
        }
    }

    /// removes transaction from all indexes
    fn index_remove(&mut self, txn: &MempoolTransaction) {
        counters::CORE_MEMPOOL_REMOVED_TXNS.inc();
        self.system_ttl_index.remove(&txn);
        self.expiration_time_index.remove(&txn);
        self.priority_index.remove(&txn);
        self.timeline_index.remove(&txn);
        self.parking_lot_index.remove(&txn);
        self.track_indices();
    }

    /// Read `count` transactions from timeline since `timeline_id`
    /// Returns block of transactions and new last_timeline_id
    pub(crate) fn read_timeline(
        &mut self,
        timeline_id: u64,
        count: usize,
    ) -> (Vec<SignedTransaction>, u64) {
        let mut batch = vec![];
        let mut last_timeline_id = timeline_id;
        for (address, sequence_number) in self.timeline_index.read_timeline(timeline_id, count) {
            if let Some(txn) = self
                .transactions
                .get_mut(&address)
                .and_then(|txns| txns.get(&sequence_number))
            {
                batch.push(txn.txn.clone());
                if let TimelineState::Ready(timeline_id) = txn.timeline_state {
                    last_timeline_id = timeline_id;
                }
            }
        }
        (batch, last_timeline_id)
    }

    pub(crate) fn timeline_range(&mut self, start_id: u64, end_id: u64) -> Vec<SignedTransaction> {
        self.timeline_index
            .timeline_range(start_id, end_id)
            .iter()
            .filter_map(|(account, sequence_number)| {
                self.transactions
                    .get(account)
                    .and_then(|txns| txns.get(sequence_number))
                    .map(|txn| txn.txn.clone())
            })
            .collect()
    }

    /// GC old transactions
    pub(crate) fn gc_by_system_ttl(
        &mut self,
        metrics_cache: &TtlCache<(AccountAddress, u64), SystemTime>,
    ) {
        let now = diem_infallible::duration_since_epoch();

        self.gc(now, true, metrics_cache);
    }

    /// GC old transactions based on client-specified expiration time
    pub(crate) fn gc_by_expiration_time(
        &mut self,
        block_time: Duration,
        metrics_cache: &TtlCache<(AccountAddress, u64), SystemTime>,
    ) {
        self.gc(block_time, false, metrics_cache);
    }

    fn gc(
        &mut self,
        now: Duration,
        by_system_ttl: bool,
        metrics_cache: &TtlCache<(AccountAddress, u64), SystemTime>,
    ) {
        let (metric_label, index, log_event) = if by_system_ttl {
            (
                counters::GC_SYSTEM_TTL_LABEL,
                &mut self.system_ttl_index,
                LogEvent::SystemTTLExpiration,
            )
        } else {
            (
                counters::GC_CLIENT_EXP_LABEL,
                &mut self.expiration_time_index,
                LogEvent::ClientExpiration,
            )
        };
        counters::CORE_MEMPOOL_GC_EVENT_COUNT
            .with_label_values(&[metric_label])
            .inc();

        let mut gc_txns = index.gc(now);
        // sort the expired txns by order of sequence number per account
        gc_txns.sort_by_key(|key| (key.address, key.sequence_number));
        let mut gc_iter = gc_txns.iter().peekable();

        let mut gc_txns_log = TxnsLog::new();
        while let Some(key) = gc_iter.next() {
            if let Some(txns) = self.transactions.get_mut(&key.address) {
                let park_range_start = Bound::Excluded(key.sequence_number);
                let park_range_end = gc_iter
                    .peek()
                    .filter(|next_key| key.address == next_key.address)
                    .map_or(Bound::Unbounded, |next_key| {
                        Bound::Excluded(next_key.sequence_number)
                    });
                // mark all following txns as non-ready, i.e. park them
                for (_, t) in txns.range((park_range_start, park_range_end)) {
                    self.parking_lot_index.insert(&t);
                    self.priority_index.remove(&t);
                    self.timeline_index.remove(&t);
                }
                if let Some(txn) = txns.remove(&key.sequence_number) {
                    let is_active = self.priority_index.contains(&txn);
                    let status = if is_active {
                        counters::GC_ACTIVE_TXN_LABEL
                    } else {
                        counters::GC_PARKED_TXN_LABEL
                    };
                    let account = txn.get_sender();
                    let sequence_number = txn.get_sequence_number();
                    gc_txns_log.add_with_status(account, sequence_number, status);
                    if let Some(&creation_time) = metrics_cache.get(&(account, sequence_number)) {
                        if let Ok(time_delta) = SystemTime::now().duration_since(creation_time) {
                            counters::CORE_MEMPOOL_GC_LATENCY
                                .with_label_values(&[metric_label, status])
                                .observe(time_delta.as_secs_f64());
                        }
                    }

                    // remove txn
                    self.index_remove(&txn);
                }
            }
        }

        debug!(LogSchema::event_log(LogEntry::GCRemoveTxns, log_event).txns(gc_txns_log));
        self.track_indices();
    }

    pub(crate) fn iter_queue(&self) -> PriorityQueueIter {
        self.priority_index.iter()
    }

    pub(crate) fn gen_snapshot(
        &self,
        metrics_cache: &TtlCache<(AccountAddress, u64), SystemTime>,
    ) -> TxnsLog {
        let mut txns_log = TxnsLog::new();
        for (account, txns) in self.transactions.iter() {
            for (seq_num, _txn) in txns.iter() {
                let status = if self.parking_lot_index.contains(account, seq_num) {
                    "parked"
                } else {
                    "ready"
                };
                let timestamp = metrics_cache.get(&(*account, *seq_num)).cloned();
                txns_log.add_full_metadata(*account, *seq_num, status, timestamp);
            }
        }
        txns_log
    }

    #[cfg(test)]
    pub(crate) fn get_parking_lot_size(&self) -> usize {
        self.parking_lot_index.size()
    }
}
