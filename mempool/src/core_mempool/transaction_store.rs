// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{
        index::{
            AccountTransactions, ParkingLotIndex, PriorityIndex, PriorityQueueIter, TTLIndex,
            TimelineIndex,
        },
        transaction::{MempoolAddTransactionStatus, MempoolTransaction, TimelineState},
    },
    proto::shared::mempool_status::MempoolAddTransactionStatusCode,
    OP_COUNTERS,
};
use config::config::MempoolConfig;
use std::{
    collections::HashMap,
    ops::Bound,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use types::{account_address::AccountAddress, transaction::SignedTransaction};

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
        if let Some(txns) = self.transactions.get(&address) {
            if let Some(txn) = txns.get(&sequence_number) {
                return Some(txn.txn.clone());
            }
        }
        None
    }

    /// insert transaction into TransactionStore
    /// performs validation checks and updates indexes
    pub(crate) fn insert(
        &mut self,
        txn: MempoolTransaction,
        current_sequence_number: u64,
    ) -> MempoolAddTransactionStatus {
        let (is_update, status) = self.check_for_update(&txn);
        if is_update {
            return status;
        }
        if self.check_if_full() {
            return MempoolAddTransactionStatus::new(
                MempoolAddTransactionStatusCode::MempoolIsFull,
                format!(
                    "mempool size: {}, capacity: {}",
                    self.system_ttl_index.size(),
                    self.capacity,
                ),
            );
        }

        let address = txn.get_sender();
        let sequence_number = txn.get_sequence_number();

        self.transactions
            .entry(address)
            .or_insert_with(AccountTransactions::new);

        if let Some(txns) = self.transactions.get_mut(&address) {
            // capacity check
            if txns.len() >= self.capacity_per_user {
                return MempoolAddTransactionStatus::new(
                    MempoolAddTransactionStatusCode::TooManyTransactions,
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
            OP_COUNTERS.set("txn.system_ttl_index", self.system_ttl_index.size());
        }
        self.process_ready_transactions(&address, current_sequence_number);
        MempoolAddTransactionStatus::new(MempoolAddTransactionStatusCode::Valid, "".to_string())
    }

    /// Check whether the queue size >= threshold in config.
    pub(crate) fn health_check(&self) -> bool {
        self.system_ttl_index.size() <= self.capacity
    }

    /// checks if Mempool is full
    /// If it's full, tries to free some space by evicting transactions from ParkingLot
    fn check_if_full(&mut self) -> bool {
        if self.system_ttl_index.size() >= self.capacity {
            // try to free some space in Mempool from ParkingLot
            if let Some((address, sequence_number)) = self.parking_lot_index.pop() {
                if let Some(txns) = self.transactions.get_mut(&address) {
                    if let Some(txn) = txns.remove(&sequence_number) {
                        self.index_remove(&txn);
                    }
                }
            }
        }
        self.system_ttl_index.size() >= self.capacity
    }

    /// check if transaction is already present in Mempool
    /// e.g. given request is update
    /// we allow increase in gas price to speed up process
    fn check_for_update(
        &mut self,
        txn: &MempoolTransaction,
    ) -> (bool, MempoolAddTransactionStatus) {
        let mut is_update = false;
        let mut status = MempoolAddTransactionStatus::new(
            MempoolAddTransactionStatusCode::Valid,
            "".to_string(),
        );

        if let Some(txns) = self.transactions.get_mut(&txn.get_sender()) {
            if let Some(current_version) = txns.get_mut(&txn.get_sequence_number()) {
                is_update = true;
                // TODO: do we need to ensure the rest of content hasn't changed
                if txn.get_gas_price() <= current_version.get_gas_price() {
                    status = MempoolAddTransactionStatus::new(
                        MempoolAddTransactionStatusCode::InvalidUpdate,
                        format!(
                            "txn gas price: {}, current_version gas price: {}",
                            txn.get_gas_price(),
                            current_version.get_gas_price(),
                        ),
                    );
                } else {
                    self.priority_index.remove(&current_version);
                    current_version.txn = txn.txn.clone();
                    self.priority_index.insert(&current_version);
                }
            }
        }
        (is_update, status)
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
                sequence_number += 1;
            }
            for (_, txn) in txns.range_mut((Bound::Excluded(sequence_number), Bound::Unbounded)) {
                match txn.timeline_state {
                    TimelineState::Ready(_) => {}
                    _ => {
                        self.parking_lot_index.insert(&txn);
                    }
                }
            }
        }
    }

    /// handles transaction commit
    /// it includes deletion of all transactions with sequence number <= `sequence_number`
    /// and potential promotion of sequential txns to PriorityIndex/TimelineIndex
    pub(crate) fn commit_transaction(&mut self, account: &AccountAddress, sequence_number: u64) {
        if let Some(txns) = self.transactions.get_mut(&account) {
            // remove all previous seq number transactions for this account
            // This can happen if transactions are sent to multiple nodes and one of
            // nodes has sent the transaction to consensus but this node still has the
            // transaction sitting in mempool
            let mut active = txns.split_off(&(sequence_number + 1));
            let txns_for_removal = txns.clone();
            txns.clear();
            txns.append(&mut active);

            for transaction in txns_for_removal.values() {
                self.index_remove(transaction);
            }
        }
        self.process_ready_transactions(account, sequence_number + 1);
    }

    /// removes transaction from all indexes
    fn index_remove(&mut self, txn: &MempoolTransaction) {
        self.system_ttl_index.remove(&txn);
        self.expiration_time_index.remove(&txn);
        self.priority_index.remove(&txn);
        self.timeline_index.remove(&txn);
        self.parking_lot_index.remove(&txn);
        OP_COUNTERS.set("txn.system_ttl_index", self.system_ttl_index.size());
    }

    /// returns gas amount required to process all transactions for given account
    pub(crate) fn get_required_balance(&mut self, address: &AccountAddress) -> u64 {
        match self.transactions.get_mut(&address) {
            Some(txns) => txns.iter().fold(0, |acc, (_, txn)| {
                acc + txn.txn.gas_unit_price() * txn.gas_amount
            }),
            None => 0,
        }
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
            if let Some(txns) = self.transactions.get_mut(&address) {
                if let Some(txn) = txns.get(&sequence_number) {
                    batch.push(txn.txn.clone());
                    if let TimelineState::Ready(timeline_id) = txn.timeline_state {
                        last_timeline_id = timeline_id;
                    }
                }
            }
        }
        (batch, last_timeline_id)
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

        for key in index.gc(now) {
            if let Some(txns) = self.transactions.get_mut(&key.address) {
                // mark all following transactions as non-ready
                for (_, t) in txns.range((Bound::Excluded(key.sequence_number), Bound::Unbounded)) {
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
        OP_COUNTERS.set("txn.system_ttl_index", self.system_ttl_index.size());
    }

    pub(crate) fn iter_queue(&self) -> PriorityQueueIter {
        self.priority_index.iter()
    }
}
