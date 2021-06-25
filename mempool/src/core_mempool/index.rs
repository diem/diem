// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// This module provides various indexes used by Mempool.
use crate::core_mempool::transaction::{MempoolTransaction, TimelineState};
use crate::{
    counters,
    logging::{LogEntry, LogSchema},
};
use diem_logger::prelude::*;
use diem_types::{
    account_address::AccountAddress, account_config::AccountSequenceNumber,
    transaction::GovernanceRole,
};
use rand::seq::SliceRandom;
use std::{
    cmp::Ordering,
    collections::{btree_set::Iter, BTreeMap, BTreeSet, HashMap},
    iter::Rev,
    ops::Bound,
    time::Duration,
};

pub type AccountTransactions = BTreeMap<u64, MempoolTransaction>;

/// PriorityIndex represents the main Priority Queue in Mempool.
/// It's used to form the transaction block for Consensus.
/// Transactions are ordered by gas price. Second level ordering is done by expiration time.
///
/// We don't store the full content of transactions in the index.
/// Instead we use `OrderedQueueKey` - logical reference to the transaction in the main store.
pub struct PriorityIndex {
    data: BTreeSet<OrderedQueueKey>,
}

pub type PriorityQueueIter<'a> = Rev<Iter<'a, OrderedQueueKey>>;

impl PriorityIndex {
    pub(crate) fn new() -> Self {
        Self {
            data: BTreeSet::new(),
        }
    }

    pub(crate) fn insert(&mut self, txn: &MempoolTransaction) {
        self.data.insert(self.make_key(&txn));
    }

    pub(crate) fn remove(&mut self, txn: &MempoolTransaction) {
        self.data.remove(&self.make_key(&txn));
    }

    pub(crate) fn contains(&self, txn: &MempoolTransaction) -> bool {
        self.data.contains(&self.make_key(txn))
    }

    fn make_key(&self, txn: &MempoolTransaction) -> OrderedQueueKey {
        OrderedQueueKey {
            gas_ranking_score: txn.ranking_score,
            expiration_time: txn.expiration_time,
            address: txn.get_sender(),
            sequence_number: txn.get_sequence_number(),
            governance_role: txn.governance_role,
            seqno_type: txn.seqno_type,
        }
    }

    pub(crate) fn iter(&self) -> PriorityQueueIter {
        self.data.iter().rev()
    }

    pub(crate) fn size(&self) -> usize {
        self.data.len()
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct OrderedQueueKey {
    pub gas_ranking_score: u64,
    pub expiration_time: Duration,
    pub address: AccountAddress,
    pub sequence_number: u64,
    pub governance_role: GovernanceRole,
    pub seqno_type: AccountSequenceNumber,
}

impl PartialOrd for OrderedQueueKey {
    fn partial_cmp(&self, other: &OrderedQueueKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedQueueKey {
    fn cmp(&self, other: &OrderedQueueKey) -> Ordering {
        match self
            .governance_role
            .priority()
            .cmp(&other.governance_role.priority())
        {
            Ordering::Equal => {}
            ordering => return ordering,
        }
        match self.gas_ranking_score.cmp(&other.gas_ranking_score) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
        match self.expiration_time.cmp(&other.expiration_time).reverse() {
            Ordering::Equal => {}
            ordering => return ordering,
        }
        match self.address.cmp(&other.address) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
        self.sequence_number.cmp(&other.sequence_number).reverse()
    }
}

/// TTLIndex is used to perform garbage collection of old transactions in Mempool.
/// Periodically separate GC-like job queries this index to find out transactions that have to be
/// removed. Index is represented as `BTreeSet<TTLOrderingKey>`, where `TTLOrderingKey`
/// is a logical reference to TxnInfo.
/// Index is ordered by `TTLOrderingKey::expiration_time`.
pub struct TTLIndex {
    data: BTreeSet<TTLOrderingKey>,
    get_expiration_time: Box<dyn Fn(&MempoolTransaction) -> Duration + Send + Sync>,
}

impl TTLIndex {
    pub(crate) fn new<F>(get_expiration_time: Box<F>) -> Self
    where
        F: Fn(&MempoolTransaction) -> Duration + 'static + Send + Sync,
    {
        Self {
            data: BTreeSet::new(),
            get_expiration_time,
        }
    }

    pub(crate) fn insert(&mut self, txn: &MempoolTransaction) {
        self.data.insert(self.make_key(&txn));
    }

    pub(crate) fn remove(&mut self, txn: &MempoolTransaction) {
        self.data.remove(&self.make_key(&txn));
    }

    /// Garbage collect all old transactions.
    pub(crate) fn gc(&mut self, now: Duration) -> Vec<TTLOrderingKey> {
        let ttl_key = TTLOrderingKey {
            expiration_time: now,
            address: AccountAddress::ZERO,
            sequence_number: 0,
        };

        let mut active = self.data.split_off(&ttl_key);
        let ttl_transactions = self.data.iter().cloned().collect();
        self.data.clear();
        self.data.append(&mut active);
        ttl_transactions
    }

    fn make_key(&self, txn: &MempoolTransaction) -> TTLOrderingKey {
        TTLOrderingKey {
            expiration_time: (self.get_expiration_time)(txn),
            address: txn.get_sender(),
            sequence_number: txn.get_sequence_number(),
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.data.len()
    }
}

#[allow(clippy::derive_ord_xor_partial_ord)]
#[derive(Eq, PartialEq, PartialOrd, Clone, Debug)]
pub struct TTLOrderingKey {
    pub expiration_time: Duration,
    pub address: AccountAddress,
    pub sequence_number: u64,
}

/// Be very careful with this, to not break the partial ordering.
/// See:  https://rust-lang.github.io/rust-clippy/master/index.html#derive_ord_xor_partial_ord
#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for TTLOrderingKey {
    fn cmp(&self, other: &TTLOrderingKey) -> Ordering {
        match self.expiration_time.cmp(&other.expiration_time) {
            Ordering::Equal => {
                (&self.address, self.sequence_number).cmp(&(&other.address, other.sequence_number))
            }
            ordering => ordering,
        }
    }
}

/// TimelineIndex is an ordered log of all transactions that are "ready" for broadcast.
/// We only add a transaction to the index if it has a chance to be included in the next consensus
/// block (which means its status is != NotReady or its sequential to another "ready" transaction).
///
/// It's represented as Map <timeline_id, (Address, sequence_number)>, where timeline_id is auto
/// increment unique id of "ready" transaction in local Mempool. (Address, sequence_number) is a
/// logical reference to transaction content in main storage.
pub struct TimelineIndex {
    timeline_id: u64,
    timeline: BTreeMap<u64, (AccountAddress, u64)>,
}

impl TimelineIndex {
    pub(crate) fn new() -> Self {
        Self {
            timeline_id: 1,
            timeline: BTreeMap::new(),
        }
    }

    /// Read all transactions from the timeline since <timeline_id>.
    pub(crate) fn read_timeline(
        &mut self,
        timeline_id: u64,
        count: usize,
    ) -> Vec<(AccountAddress, u64)> {
        let mut batch = vec![];
        for (_id, &(address, sequence_number)) in self
            .timeline
            .range((Bound::Excluded(timeline_id), Bound::Unbounded))
        {
            batch.push((address, sequence_number));
            if batch.len() == count {
                break;
            }
        }
        batch
    }

    /// Read transactions from the timeline from `start_id` (exclusive) to `end_id` (inclusive).
    pub(crate) fn timeline_range(
        &mut self,
        start_id: u64,
        end_id: u64,
    ) -> Vec<(AccountAddress, u64)> {
        self.timeline
            .range((Bound::Excluded(start_id), Bound::Included(end_id)))
            .map(|(_idx, txn)| txn)
            .cloned()
            .collect()
    }

    pub(crate) fn insert(&mut self, txn: &mut MempoolTransaction) {
        self.timeline.insert(
            self.timeline_id,
            (txn.get_sender(), txn.get_sequence_number()),
        );
        txn.timeline_state = TimelineState::Ready(self.timeline_id);
        self.timeline_id += 1;
    }

    pub(crate) fn remove(&mut self, txn: &MempoolTransaction) {
        if let TimelineState::Ready(timeline_id) = txn.timeline_state {
            self.timeline.remove(&timeline_id);
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.timeline.len()
    }
}

/// ParkingLotIndex keeps track of "not_ready" transactions, e.g., transactions that
/// can't be included in the next block because their sequence number is too high.
/// We keep a separate index to be able to efficiently evict them when Mempool is full.
pub struct ParkingLotIndex {
    // DS invariants:
    // 1. for each entry (account, txns) in `data`, `txns` is never empty
    // 2. for all accounts, data.get(account_indices.get(`account`)) == (account, sequence numbers of account's txns)
    data: Vec<(AccountAddress, BTreeSet<u64>)>,
    account_indices: HashMap<AccountAddress, usize>,
    size: usize,
}

impl ParkingLotIndex {
    pub(crate) fn new() -> Self {
        Self {
            data: vec![],
            account_indices: HashMap::new(),
            size: 0,
        }
    }

    pub(crate) fn insert(&mut self, txn: &MempoolTransaction) {
        let sender = &txn.txn.sender();
        let sequence_number = txn.txn.sequence_number();
        let is_new_entry = match self.account_indices.get(sender) {
            Some(index) => {
                if let Some((_account, seq_nums)) = self.data.get_mut(*index) {
                    seq_nums.insert(sequence_number)
                } else {
                    counters::CORE_MEMPOOL_INVARIANT_VIOLATION_COUNT.inc();
                    error!(
                        LogSchema::new(LogEntry::InvariantViolated),
                        "Parking lot invariant violated: for account {}, account index exists but missing entry in data",
                        sender
                    );
                    return;
                }
            }
            None => {
                let seq_nums = [sequence_number].iter().cloned().collect::<BTreeSet<_>>();
                self.data.push((*sender, seq_nums));
                self.account_indices.insert(*sender, self.data.len() - 1);
                true
            }
        };
        if is_new_entry {
            self.size += 1;
        }
    }

    pub(crate) fn remove(&mut self, txn: &MempoolTransaction) {
        let sender = &txn.txn.sender();
        if let Some(index) = self.account_indices.get(sender).cloned() {
            if let Some((_account, txns)) = self.data.get_mut(index) {
                if txns.remove(&txn.txn.sequence_number()) {
                    self.size -= 1;
                }

                // maintain DS invariant
                if txns.is_empty() {
                    // remove account with no more txns
                    self.data.swap_remove(index);
                    self.account_indices.remove(sender);

                    // update DS for account that was swapped in `swap_remove`
                    if let Some((swapped_account, _)) = self.data.get(index) {
                        self.account_indices.insert(*swapped_account, index);
                    }
                }
            }
        }
    }

    pub(crate) fn contains(&self, account: &AccountAddress, seq_num: &u64) -> bool {
        self.account_indices
            .get(&account)
            .and_then(|idx| self.data.get(*idx))
            .map_or(false, |(_account, txns)| txns.contains(seq_num))
    }

    /// Returns a random "non-ready" transaction (with highest sequence number for that account).
    pub(crate) fn get_poppable(&mut self) -> Option<TxnPointer> {
        let mut rng = rand::thread_rng();
        self.data
            .choose(&mut rng)
            .and_then(|(sender, txns)| txns.iter().rev().next().map(|seq_num| (*sender, *seq_num)))
    }

    pub(crate) fn size(&self) -> usize {
        self.size
    }
}

/// Logical pointer to `MempoolTransaction`.
/// Includes Account's address and transaction sequence number.
pub type TxnPointer = (AccountAddress, u64);

impl From<&MempoolTransaction> for TxnPointer {
    fn from(transaction: &MempoolTransaction) -> Self {
        (transaction.get_sender(), transaction.get_sequence_number())
    }
}

impl From<&OrderedQueueKey> for TxnPointer {
    fn from(key: &OrderedQueueKey) -> Self {
        (key.address, key.sequence_number)
    }
}
