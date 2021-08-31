// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tasks that are executed by coordinators (short-lived compared to coordinators)

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    counters,
    logging::{LogEntry, LogEvent, LogSchema},
    network::MempoolSyncMsg,
    shared_mempool::types::{
        notify_subscribers, ScheduledBroadcast, SharedMempool, SharedMempoolNotification,
        SubmissionStatusBundle, TransactionSummary,
    },
    ConsensusRequest, ConsensusResponse, SubmissionStatus,
};
use anyhow::Result;
use diem_config::config::PeerNetworkId;
use diem_infallible::{Mutex, RwLock};
use diem_logger::prelude::*;
use diem_metrics::HistogramTimer;
use diem_types::{
    mempool_status::{MempoolStatus, MempoolStatusCode},
    on_chain_config::OnChainConfigPayload,
    transaction::SignedTransaction,
    vm_status::DiscardedVMStatus,
};
use futures::{channel::oneshot, stream::FuturesUnordered};
use rayon::prelude::*;
use short_hex_str::AsShortHexStr;
use std::{
    cmp,
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::runtime::Handle;
use vm_validator::vm_validator::{get_account_sequence_number, TransactionValidation};

// ============================== //
//  broadcast_coordinator tasks  //
// ============================== //

/// Attempts broadcast to `peer` and schedules the next broadcast.
pub(crate) fn execute_broadcast<V>(
    peer: PeerNetworkId,
    backoff: bool,
    smp: &mut SharedMempool<V>,
    scheduled_broadcasts: &mut FuturesUnordered<ScheduledBroadcast>,
    executor: Handle,
) where
    V: TransactionValidation,
{
    let peer_manager = &smp.peer_manager.clone();
    peer_manager.execute_broadcast(peer.clone(), backoff, smp);
    let schedule_backoff = peer_manager.is_backoff_mode(&peer);

    let interval_ms = if schedule_backoff {
        smp.config.shared_mempool_backoff_interval_ms
    } else {
        smp.config.shared_mempool_tick_interval_ms
    };

    scheduled_broadcasts.push(ScheduledBroadcast::new(
        Instant::now() + Duration::from_millis(interval_ms),
        peer,
        schedule_backoff,
        executor,
    ))
}

// =============================== //
// Tasks processing txn submission //
// =============================== //

/// Processes transactions directly submitted by client.
pub(crate) async fn process_client_transaction_submission<V>(
    smp: SharedMempool<V>,
    transaction: SignedTransaction,
    callback: oneshot::Sender<Result<SubmissionStatus>>,
    timer: HistogramTimer,
) where
    V: TransactionValidation,
{
    timer.stop_and_record();
    let _timer =
        counters::process_txn_submit_latency_timer(counters::CLIENT_LABEL, counters::CLIENT_LABEL);
    let statuses =
        process_incoming_transactions(&smp, vec![transaction], TimelineState::NotReady).await;
    log_txn_process_results(&statuses, None);

    if let Some(status) = statuses.get(0) {
        if callback.send(Ok(status.1.clone())).is_err() {
            error!(LogSchema::event_log(
                LogEntry::JsonRpc,
                LogEvent::CallbackFail
            ));
            counters::CLIENT_CALLBACK_FAIL.inc();
        }
    }
}

/// Processes transactions from other nodes.
pub(crate) async fn process_transaction_broadcast<V>(
    mut smp: SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    request_id: Vec<u8>,
    timeline_state: TimelineState,
    peer: PeerNetworkId,
    timer: HistogramTimer,
) where
    V: TransactionValidation,
{
    timer.stop_and_record();
    let _timer = counters::process_txn_submit_latency_timer(
        peer.raw_network_id().as_str(),
        peer.peer_id().short_str().as_str(),
    );
    let results = process_incoming_transactions(&smp, transactions.clone(), timeline_state).await;
    log_txn_process_results(&results, Some(peer.clone()));

    let ack_response = gen_ack_response(request_id, results, &peer);
    let network_sender = smp
        .network_senders
        .get_mut(&peer.network_id())
        .expect("[shared mempool] missing network sender");
    if let Err(e) = network_sender.send_to(peer.peer_id(), ack_response) {
        counters::network_send_fail_inc(counters::ACK_TXNS);
        error!(
            LogSchema::event_log(LogEntry::BroadcastACK, LogEvent::NetworkSendFail)
                .peer(&peer)
                .error(&e.into())
        );
        return;
    }
    notify_subscribers(SharedMempoolNotification::ACK, &smp.subscribers);
}

fn gen_ack_response(
    request_id: Vec<u8>,
    results: Vec<SubmissionStatusBundle>,
    peer: &PeerNetworkId,
) -> MempoolSyncMsg {
    let mut backoff = false;
    let mut retry = false;
    for r in results.into_iter() {
        let submission_status = r.1;
        if submission_status.0.code == MempoolStatusCode::MempoolIsFull {
            backoff = true;
        }
        if is_txn_retryable(submission_status) {
            retry = true;
        }

        if backoff && retry {
            break;
        }
    }

    update_ack_counter(peer, counters::SENT_LABEL, retry, backoff);
    MempoolSyncMsg::BroadcastTransactionsResponse {
        request_id,
        retry,
        backoff,
    }
}

pub(crate) fn update_ack_counter(
    peer: &PeerNetworkId,
    direction_label: &str,
    retry: bool,
    backoff: bool,
) {
    if retry {
        counters::shared_mempool_ack_inc(peer, direction_label, counters::RETRY_BROADCAST_LABEL);
    }
    if backoff {
        counters::shared_mempool_ack_inc(
            peer,
            direction_label,
            counters::BACKPRESSURE_BROADCAST_LABEL,
        );
    }
}

fn is_txn_retryable(result: SubmissionStatus) -> bool {
    result.0.code == MempoolStatusCode::MempoolIsFull
}

/// Submits a list of SignedTransaction to the local mempool
/// and returns a vector containing AdmissionControlStatus.
pub(crate) async fn process_incoming_transactions<V>(
    smp: &SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    timeline_state: TimelineState,
) -> Vec<SubmissionStatusBundle>
where
    V: TransactionValidation,
{
    let mut statuses = vec![];

    let start_storage_read = Instant::now();
    // Track latency: fetching seq number
    let seq_numbers = transactions
        .par_iter()
        .map(|t| {
            get_account_sequence_number(smp.db.as_ref(), t.sender()).map_err(|e| {
                error!(LogSchema::new(LogEntry::DBError).error(&e));
                counters::DB_ERROR.inc();
                e
            })
        })
        .collect::<Vec<_>>();
    // Track latency for storage read fetching sequence number
    let storage_read_latency = start_storage_read.elapsed();
    counters::PROCESS_TXN_BREAKDOWN_LATENCY
        .with_label_values(&[counters::FETCH_SEQ_NUM_LABEL])
        .observe(storage_read_latency.as_secs_f64() / transactions.len() as f64);

    let transactions: Vec<_> = transactions
        .into_iter()
        .enumerate()
        .filter_map(|(idx, t)| {
            if let Ok(crsn_or_seqno) = seq_numbers[idx] {
                if t.sequence_number() >= crsn_or_seqno.min_seq() {
                    return Some((t, crsn_or_seqno));
                } else {
                    statuses.push((
                        t,
                        (
                            MempoolStatus::new(MempoolStatusCode::VmError),
                            Some(DiscardedVMStatus::SEQUENCE_NUMBER_TOO_OLD),
                        ),
                    ));
                }
            } else {
                // Failed to get transaction
                statuses.push((
                    t,
                    (
                        MempoolStatus::new(MempoolStatusCode::VmError),
                        Some(DiscardedVMStatus::RESOURCE_DOES_NOT_EXIST),
                    ),
                ));
            }
            None
        })
        .collect();

    // Track latency: VM validation
    let vm_validation_timer = counters::PROCESS_TXN_BREAKDOWN_LATENCY
        .with_label_values(&[counters::VM_VALIDATION_LABEL])
        .start_timer();
    let validation_results = transactions
        .par_iter()
        .map(|t| smp.validator.read().validate_transaction(t.0.clone()))
        .collect::<Vec<_>>();
    vm_validation_timer.stop_and_record();

    {
        let mut mempool = smp.mempool.lock();
        for (idx, (transaction, crsn_or_seqno)) in transactions.into_iter().enumerate() {
            if let Ok(validation_result) = &validation_results[idx] {
                match validation_result.status() {
                    None => {
                        let gas_amount = transaction.max_gas_amount();
                        let ranking_score = validation_result.score();
                        let governance_role = validation_result.governance_role();
                        let mempool_status = mempool.add_txn(
                            transaction.clone(),
                            gas_amount,
                            ranking_score,
                            crsn_or_seqno,
                            timeline_state,
                            governance_role,
                        );
                        statuses.push((transaction, (mempool_status, None)));
                    }
                    Some(validation_status) => {
                        statuses.push((
                            transaction.clone(),
                            (
                                MempoolStatus::new(MempoolStatusCode::VmError),
                                Some(validation_status),
                            ),
                        ));
                    }
                }
            }
        }
    }
    notify_subscribers(SharedMempoolNotification::NewTransactions, &smp.subscribers);
    statuses
}

fn log_txn_process_results(results: &[SubmissionStatusBundle], sender: Option<PeerNetworkId>) {
    let (network, sender) = match sender {
        Some(peer) => (
            peer.raw_network_id().to_string(),
            peer.peer_id().to_string(),
        ),
        None => (
            counters::CLIENT_LABEL.to_string(),
            counters::CLIENT_LABEL.to_string(),
        ),
    };
    for (txn, (mempool_status, maybe_vm_status)) in results.iter() {
        if let Some(vm_status) = maybe_vm_status {
            trace!(
                SecurityEvent::InvalidTransactionMempool,
                failed_transaction = txn,
                vm_status = vm_status,
                sender = sender,
            );
            counters::shared_mempool_transactions_processed_inc(
                counters::VM_VALIDATION_LABEL,
                &network,
                &sender,
            );
            continue;
        }
        match mempool_status.code {
            MempoolStatusCode::Accepted => counters::shared_mempool_transactions_processed_inc(
                counters::SUCCESS_LABEL,
                &network,
                &sender,
            ),
            _ => counters::shared_mempool_transactions_processed_inc(
                &mempool_status.code.to_string(),
                &network,
                &sender,
            ),
        }
    }
}

// ================================= //
// intra-node communication handlers //
// ================================= //

pub(crate) async fn process_consensus_request(mempool: &Mutex<CoreMempool>, req: ConsensusRequest) {
    // Start latency timer
    let start_time = Instant::now();
    debug!(LogSchema::event_log(LogEntry::Consensus, LogEvent::Received).consensus_msg(&req));

    let (resp, callback, counter_label) = match req {
        ConsensusRequest::GetBlockRequest(max_block_size, transactions, callback) => {
            let exclude_transactions: HashSet<TxnPointer> = transactions
                .iter()
                .map(|txn| (txn.sender, txn.sequence_number))
                .collect();
            let mut txns;
            {
                let mut mempool = mempool.lock();
                // gc before pulling block as extra protection against txns that may expire in consensus
                // Note: this gc operation relies on the fact that consensus uses the system time to determine block timestamp
                let curr_time = diem_infallible::duration_since_epoch();
                mempool.gc_by_expiration_time(curr_time);
                let block_size = cmp::max(max_block_size, 1);
                txns = mempool.get_block(block_size, exclude_transactions);
            }
            counters::mempool_service_transactions(counters::GET_BLOCK_LABEL, txns.len());
            txns.len();
            let pulled_block = txns.drain(..).map(SignedTransaction::into).collect();

            (
                ConsensusResponse::GetBlockResponse(pulled_block),
                callback,
                counters::GET_BLOCK_LABEL,
            )
        }
        ConsensusRequest::RejectNotification(transactions, callback) => {
            counters::mempool_service_transactions(
                counters::COMMIT_CONSENSUS_LABEL,
                transactions.len(),
            );
            commit_txns(mempool, transactions, 0, true).await;
            (
                ConsensusResponse::CommitResponse(),
                callback,
                counters::COMMIT_CONSENSUS_LABEL,
            )
        }
    };
    // Send back to callback
    let result = if callback.send(Ok(resp)).is_err() {
        error!(LogSchema::event_log(
            LogEntry::Consensus,
            LogEvent::CallbackFail
        ));
        counters::REQUEST_FAIL_LABEL
    } else {
        counters::REQUEST_SUCCESS_LABEL
    };
    let latency = start_time.elapsed();
    counters::mempool_service_latency(counter_label, result, latency);
}

pub async fn commit_txns(
    mempool: &Mutex<CoreMempool>,
    transactions: Vec<TransactionSummary>,
    block_timestamp_usecs: u64,
    is_rejected: bool,
) {
    let mut pool = mempool.lock();

    for transaction in transactions {
        pool.remove_transaction(
            &transaction.sender,
            transaction.sequence_number,
            is_rejected,
        );
    }

    if block_timestamp_usecs > 0 {
        pool.gc_by_expiration_time(Duration::from_micros(block_timestamp_usecs));
    }
}

/// Processes on-chain reconfiguration notification.
pub(crate) async fn process_config_update<V>(
    config_update: OnChainConfigPayload,
    validator: Arc<RwLock<V>>,
) where
    V: TransactionValidation,
{
    info!(
        LogSchema::event_log(LogEntry::ReconfigUpdate, LogEvent::Process)
            .reconfig_update(config_update.clone())
    );

    if let Err(e) = validator.write().restart(config_update) {
        counters::VM_RECONFIG_UPDATE_FAIL_COUNT.inc();
        error!(LogSchema::event_log(LogEntry::ReconfigUpdate, LogEvent::VMUpdateFail).error(&e));
    }
}
