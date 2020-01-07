// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Mempool is used to hold transactions that have been submitted but not yet agreed upon and
//! executed.
//!
//! **Flow**: AC sends transactions into mempool which holds them for a period of time before
//! sending them into consensus.  When a new transaction is added, Mempool shares this transaction
//! with other nodes in the system.  This is a form of “shared mempool” in that transactions between
//! mempools are shared with other validators.  This helps maintain a pseudo global ordering since
//! when a validator receives a transaction from another mempool, it will be ordered when added in
//! the ordered queue of the recipient validator. To reduce network consumption, in “shared mempool”
//! each validator is responsible for delivery of its own transactions (we don't rebroadcast
//! transactions originated on a different peer). Also we only broadcast transactions that have some
//! chance to be included in next block: their sequence number equals to the next sequence number of
//! account or sequential to it. For example, if the current sequence number for an account is 2 and
//! local mempool contains transactions with sequence numbers 2,3,4,7,8, then only transactions 2, 3
//! and 4 will be broadcast.
//!
//! Consensus pulls transactions from mempool rather than mempool pushing into consensus. This is
//! done so that while consensus is not yet ready for transactions, we keep ordering based on gas
//! and consensus can let transactions build up.  This allows for batching of transactions into a
//! single consensus block as well as prioritizing by gas price. Mempool doesn't  keep track of
//! transactions that were sent to Consensus. On each get_block request, Consensus additionally
//! sends a set of transactions that were pulled from Mempool so far but were not committed yet.
//! This is done so Mempool can be agnostic about different Consensus proposal branches.  Once a
//! transaction is fully executed and written to storage,  Consensus notifies Mempool about it which
//! later drops it from its internal state.
//!
//! **Internals**: Internally Mempool is modeled as `HashMap<AccountAddress, AccountTransactions>`
//! with various indexes built on top of it. The main index `PriorityIndex` is an ordered queue of
//! transactions that are “ready” to be included in next block(i.e. have sequence number sequential
//! to current for account). This queue is ordered by gas price so that if a client is willing to
//! pay more (than other clients) per unit of execution, then they can enter consensus earlier. Note
//! that although global ordering is maintained by gas price, for a single account, transactions are
//! ordered by sequence number.
//!
//! All transactions that are not ready to be included in the next block are part of separate
//! `ParkingLotIndex`. They will be moved to the ordered queue once some event unblocks them. For
//! example, Mempool has transaction with sequence number 4, while current sequence number for that
//! account is 3. Such transaction is considered to be “non-ready”. Then callback from Consensus
//! notifies that transaction was committed(i.e. transaction 3 was submitted to different node).
//! Such event “unblocks” local transaction and txn4 will be moved to OrderedQueue.
//!
//! Mempool only holds a limited number of transactions to prevent OOMing the system. Additionally
//! there's a limit of number of transactions per account to prevent different abuses/attacks
//!
//! Transactions in Mempool have two types of expirations: systemTTL and client-specified
//! expiration. Once we hit either of those, the transaction is removed from Mempool. SystemTTL is
//! checked periodically in the background, while the client-specified expiration is checked on
//! every Consensus commit request. We use a separate system TTL to ensure that a transaction won't
//! remain stuck in Mempool forever, even if Consensus doesn't make progress

pub mod proto;
pub use runtime::MempoolRuntime;

mod core_mempool;
mod mempool_service;
mod runtime;
mod shared_mempool;

// module op counters
use libra_metrics::OpMetrics;
use once_cell::sync::Lazy;
static OP_COUNTERS: Lazy<OpMetrics> = Lazy::new(|| OpMetrics::new_and_registered("mempool"));

#[cfg(test)]
mod unit_tests;
