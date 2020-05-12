// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{
        index::{
            AccountTransactions, ParkingLotIndex, PriorityIndex, PriorityQueueIter, TTLIndex,
            TimelineIndex,
        },
        transaction::{MempoolTransaction, TimelineState},
    },
    OP_COUNTERS,
};
use anyhow::{format_err, Result};
use libra_config::config::MempoolConfig;
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    mempool_status::{MempoolStatus, MempoolStatusCode},
    transaction::SignedTransaction,
};
use std::{
    collections::HashMap,
    ops::Bound,
    time::{Duration, SystemTime, UNIX_EPOCH},
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
                t.txn.expiration_time()
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

        if self.check_if_full() {
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
        OP_COUNTERS.set("txn.system_ttl_index", self.system_ttl_index.size());
        OP_COUNTERS.set("txn.parking_lot_index", self.parking_lot_index.size());
        OP_COUNTERS.set("txn.priority_index", self.priority_index.size());
    }

    /// checks if Mempool is full
    /// If it's full, tries to free some space by evicting transactions from ParkingLot
    fn check_if_full(&mut self) -> bool {
        if self.system_ttl_index.size() >= self.capacity {
            // try to free some space in Mempool from ParkingLot
            if let Some((address, sequence_number)) = self.parking_lot_index.pop() {
                if let Some(txn) = self
                    .transactions
                    .get_mut(&address)
                    .and_then(|txns| txns.remove(&sequence_number))
                {
                    debug!("[mempool] mempool full, evicting txn {}:{}", address, sequence_number);
                    self.index_remove(&txn);
                }
            }
        }
        self.system_ttl_index.size() >= self.capacity
    }

    /// check if transaction is already present in Mempool
    /// e.g. given request is update
    /// we allow increase in gas price to speed up process
    fn handle_gas_price_update(&mut self, txn: &MempoolTransaction) -> Result<()> {
        if let Some(txns) = self.transactions.get_mut(&txn.get_sender()) {
            if let Some(current_version) = txns.get_mut(&txn.get_sequence_number()) {
                if current_version.txn.max_gas_amount() == txn.txn.max_gas_amount()
                    && current_version.txn.payload() == txn.txn.payload()
                    && current_version.txn.expiration_time() == txn.txn.expiration_time()
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
                // TODO remove txn from parking lot index too!
                self.parking_lot_index.remove(txn);
                sequence_number += 1;
            }

            let mut parking_lot_txns = 0;
            for (_, txn) in txns.range_mut((Bound::Excluded(sequence_number), Bound::Unbounded)) {
                match txn.timeline_state {
                    TimelineState::Ready(_) => {}
                    _ => {
//                        debug!("[mempool] process ready txn - adding to parking lot index");
                        self.parking_lot_index.insert(&txn);
                        parking_lot_txns += 1;
                    }
                }
            }
            trace!("[Mempool] txns for account {:?}. Current sequence_number: {}, length: {}, parking lot: {}",
                address, current_sequence_number, txns.len(), parking_lot_txns,
            );
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

            for transaction in txns_for_removal.values() {
                self.index_remove(transaction);
            }
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
            for transaction in txns.values() {
                self.index_remove(&transaction);
            }
        }
    }

    /// removes transaction from all indexes
    fn index_remove(&mut self, txn: &MempoolTransaction) {
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

    /// Returns block of transactions with timeline id in the range `start_timeline_id` exclusive to `end_timeline_id` inclusive
    pub(crate) fn timeline_range(
        &mut self,
        start_timeline_id: u64,
        end_timeline_id: u64,
    ) -> Vec<SignedTransaction> {
        let mut batch = vec![];
        for (address, sequence_number) in self
            .timeline_index
            .range(start_timeline_id, end_timeline_id)
        {
            if let Some(txn) = self
                .transactions
                .get_mut(&address)
                .and_then(|txns| txns.get(&sequence_number))
            {
                batch.push(txn.txn.clone());
            }
        }
        batch
    }

    /// GC old transactions
    pub(crate) fn gc_by_system_ttl(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("init timestamp failure");

        self.gc(now, true);
    }

    /// GC old transactions based on client-specified expiration time
    pub(crate) fn gc_by_expiration_time(&mut self, block_time: Duration) {
        self.gc(block_time, false);
    }

    fn gc(&mut self, now: Duration, by_system_ttl: bool) {
        let (index_name, index) = if by_system_ttl {
            ("gc.system_ttl_index", &mut self.system_ttl_index)
        } else {
            ("gc.expiration_time_index", &mut self.expiration_time_index)
        };
        OP_COUNTERS.inc(index_name);
//        debug!("[mempool] gc");

        for key in index.gc(now) {
            if let Some(txns) = self.transactions.get_mut(&key.address) {
                // mark all following transactions as non-ready
                for (_, t) in txns.range((Bound::Excluded(key.sequence_number), Bound::Unbounded)) {
//                    debug!("[mempool] gc - adding to parking lot index {}:{}", t.get_sender(), t.get_sequence_number());
                    self.parking_lot_index.insert(&t);
                    self.priority_index.remove(&t);
                    self.timeline_index.remove(&t);
                }
                if let Some(txn) = txns.remove(&key.sequence_number) {
                    let is_active = self.priority_index.contains(&txn);
                    let status = if is_active { "active" } else { "parked" };
                    OP_COUNTERS.inc(&format!("{}.{}", index_name, status));
                    self.index_remove(&txn);
                }
            }
        }
        self.track_indices();
    }

    pub(crate) fn iter_queue(&self) -> PriorityQueueIter {
        self.priority_index.iter()
    }
}
