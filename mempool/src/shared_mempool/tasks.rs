// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tasks that are executed by coordinators (short-lived compared to coordinators)

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    counters,
    network::{MempoolNetworkSender, MempoolSyncMsg},
    shared_mempool::types::{
        notify_subscribers, ScheduledBroadcast, SharedMempool, SharedMempoolNotification,
    },
    CommitNotification, CommitResponse, CommittedTransaction, ConsensusRequest, ConsensusResponse,
    SubmissionStatus,
};
use anyhow::{format_err, Result};
use futures::{channel::oneshot, stream::FuturesUnordered};
use libra_config::config::PeerNetworkId;
use libra_logger::prelude::*;
use libra_types::{
    mempool_status::{MempoolStatus, MempoolStatusCode},
    on_chain_config::OnChainConfigPayload,
    transaction::SignedTransaction,
    vm_status::DiscardedVMStatus,
    PeerId,
};
use std::{
    cmp,
    collections::HashSet,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};
use tokio::runtime::Handle;
use vm_validator::vm_validator::{get_account_sequence_number, TransactionValidation};

// ============================== //
//  broadcast_coordinator tasks  //
// ============================== //

/// attempts broadcast to `peer` and schedules the next broadcast
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

pub(crate) fn send_mempool_sync_msg(
    msg: MempoolSyncMsg,
    recipient: PeerId,
    network_sender: &mut MempoolNetworkSender,
) -> Result<()> {
    // Since this is a direct-send, this will only error if the network
    // module has unexpectedly crashed or shutdown.
    network_sender.send_to(recipient, msg).map_err(|e| {
        format_err!(
            "[shared mempool] failed to direct-send mempool sync message: {}",
            e
        )
    })
}

// =============================== //
// tasks processing txn submission //
// =============================== //

/// processes transactions directly submitted by client
pub(crate) async fn process_client_transaction_submission<V>(
    smp: SharedMempool<V>,
    transaction: SignedTransaction,
    callback: oneshot::Sender<Result<SubmissionStatus>>,
) where
    V: TransactionValidation,
{
    let _timer = counters::PROCESS_TXN_SUBMISSION_LATENCY
        .with_label_values(&["client"])
        .start_timer();
    let mut statuses =
        process_incoming_transactions(&smp, vec![transaction], TimelineState::NotReady).await;
    log_txn_process_results(&statuses, None);
    let status;
    if statuses.is_empty() {
        error!("[shared mempool] missing status for client transaction submission");
        return;
    } else {
        status = statuses.remove(0);
    }

    if let Err(e) = callback
        .send(Ok(status))
        .map_err(|_| format_err!("[shared mempool] timeout on callback send to AC endpoint"))
    {
        error!("[shared mempool] failed to send back transaction submission result to AC endpoint with error: {:?}", e);
    }
}

/// processes transactions from other nodes
pub(crate) async fn process_transaction_broadcast<V>(
    mut smp: SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    request_id: String,
    timeline_state: TimelineState,
    peer: PeerNetworkId,
) where
    V: TransactionValidation,
{
    let _timer = counters::PROCESS_TXN_SUBMISSION_LATENCY
        .with_label_values(&[&peer.peer_id().to_string()])
        .start_timer();
    // process transactions and log the result
    let results = process_incoming_transactions(&smp, transactions.clone(), timeline_state).await;
    log_txn_process_results(&results, Some(peer.peer_id()));

    // we assume that the results contains a vec of the same length of transactions
    debug_assert_eq!(results.len(), transactions.len());

    // security log all the failed transactions
    let failed_transactions = results
        .iter()
        // create iterator of tuple (VM status, transaction)
        .map(|(_, maybe_vm_status)| maybe_vm_status)
        .zip(transactions.iter())
        // VM status will be set if the transaction failed
        .filter(|(maybe_vm_status, _)| maybe_vm_status.is_some());

    for (maybe_vm_status, failed_transaction) in failed_transactions {
        error!(
            SecurityEvent::InvalidTransactionMempool,
            failed_transaction = failed_transaction,
            vm_status = maybe_vm_status,
            from_peer = peer,
        );
    }

    // send back ACK
    let ack_response = gen_ack_response(request_id, results);
    let mut network_sender = smp
        .network_senders
        .get_mut(&peer.network_id())
        .expect("[shared mempool] missing network sender");
    if let Err(e) = send_mempool_sync_msg(ack_response, peer.peer_id(), &mut network_sender) {
        error!(
            "[shared mempool] failed to send ACK back to peer {:?}: {}",
            peer, e
        );
    }
    notify_subscribers(SharedMempoolNotification::ACK, &smp.subscribers);
}

fn gen_ack_response(request_id: String, results: Vec<SubmissionStatus>) -> MempoolSyncMsg {
    let mut backoff = false;
    let retry_txns = results
        .into_iter()
        .enumerate()
        .filter_map(|(idx, result)| {
            backoff = backoff || result.0.code == MempoolStatusCode::MempoolIsFull;

            if is_txn_retryable(result) {
                Some(idx as u64)
            } else {
                None
            }
        })
        .collect();

    MempoolSyncMsg::BroadcastTransactionsResponse {
        request_id,
        retry_txns,
        backoff,
    }
}

fn is_txn_retryable(result: SubmissionStatus) -> bool {
    result.0.code == MempoolStatusCode::MempoolIsFull
}

/// submits a list of SignedTransaction to the local mempool
/// and returns a vector containing AdmissionControlStatus
pub(crate) async fn process_incoming_transactions<V>(
    smp: &SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    timeline_state: TimelineState,
) -> Vec<SubmissionStatus>
where
    V: TransactionValidation,
{
    let mut statuses = vec![];

    let start_storage_read = Instant::now();
    // track latency: fetching seq number
    let seq_numbers = transactions
        .iter()
        .map(|t| get_account_sequence_number(smp.db.as_ref(), t.sender()))
        .collect::<Vec<_>>();
    // track latency for storage read fetching sequence number
    let storage_read_latency = start_storage_read.elapsed();
    counters::PROCESS_TXN_BREAKDOWN_LATENCY
        .with_label_values(&[counters::FETCH_SEQ_NUM_LABEL])
        .observe(storage_read_latency.as_secs_f64() / transactions.len() as f64);

    let transactions: Vec<_> = transactions
        .into_iter()
        .enumerate()
        .filter_map(|(idx, t)| {
            if let Ok(sequence_number) = seq_numbers[idx] {
                if t.sequence_number() >= sequence_number {
                    return Some((t, sequence_number));
                } else {
                    statuses.push((
                        MempoolStatus::new(MempoolStatusCode::VmError),
                        Some(DiscardedVMStatus::SEQUENCE_NUMBER_TOO_OLD),
                    ));
                }
            } else {
                // failed to get transaction
                statuses.push((
                    MempoolStatus::new(MempoolStatusCode::VmError),
                    Some(DiscardedVMStatus::RESOURCE_DOES_NOT_EXIST),
                ));
            }
            None
        })
        .collect();

    // track latency: VM validation
    let vm_validation_timer = counters::PROCESS_TXN_BREAKDOWN_LATENCY
        .with_label_values(&[counters::VM_VALIDATION_LABEL])
        .start_timer();
    let validation_results = transactions
        .iter()
        .map(|t| {
            smp.validator
                .read()
                .unwrap()
                .validate_transaction(t.0.clone())
        })
        .collect::<Vec<_>>();
    vm_validation_timer.stop_and_record();

    {
        let mut mempool = smp
            .mempool
            .lock()
            .expect("[shared mempool] failed to acquire mempool lock");
        for (idx, (transaction, sequence_number)) in transactions.into_iter().enumerate() {
            if let Ok(validation_result) = &validation_results[idx] {
                match validation_result.status() {
                    None => {
                        let gas_amount = transaction.max_gas_amount();
                        let ranking_score = validation_result.score();
                        let governance_role = validation_result.governance_role();
                        let mempool_status = mempool.add_txn(
                            transaction,
                            gas_amount,
                            ranking_score,
                            sequence_number,
                            timeline_state,
                            governance_role,
                        );
                        statuses.push((mempool_status, None));
                    }
                    Some(validation_status) => {
                        statuses.push((
                            MempoolStatus::new(MempoolStatusCode::VmError),
                            Some(validation_status),
                        ));
                    }
                }
            }
        }
    }
    notify_subscribers(SharedMempoolNotification::NewTransactions, &smp.subscribers);
    statuses
}

// TODO update counters to ID peers using PeerNetworkId
fn log_txn_process_results(results: &[SubmissionStatus], sender: Option<PeerId>) {
    let sender = match sender {
        Some(peer) => peer.to_string(),
        None => "client".to_string(),
    };
    for (mempool_status, vm_status) in results.iter() {
        if vm_status.is_some() {
            // log vm validation failure
            counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                .with_label_values(&["validation_failed".to_string().deref(), &sender])
                .inc();
            continue;
        }
        match mempool_status.code {
            MempoolStatusCode::Accepted => {
                counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                    .with_label_values(&["success".to_string().deref(), &sender])
                    .inc();
            }
            _ => {
                counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                    .with_label_values(&[format!("{:?}", mempool_status.code).deref(), &sender])
                    .inc();
            }
        }
    }
}

// ================================= //
// intra-node communication handlers //
// ================================= //
pub(crate) async fn process_state_sync_request(
    mempool: Arc<Mutex<CoreMempool>>,
    req: CommitNotification,
) {
    let start_time = Instant::now();
    counters::MEMPOOL_SERVICE_TXNS
        .with_label_values(&[counters::COMMIT_STATE_SYNC_LABEL])
        .observe(req.transactions.len() as f64);
    commit_txns(&mempool, req.transactions, req.block_timestamp_usecs, false).await;
    // send back to callback
    let result = if let Err(e) = req
        .callback
        .send(Ok(CommitResponse {
            msg: "".to_string(),
        }))
        .map_err(|_| {
            format_err!("[shared mempool] timeout on callback sending response to Mempool request")
        }) {
        error!(
            "[shared mempool] failed to send back CommitResponse with error: {:?}",
            e
        );
        counters::REQUEST_FAIL_LABEL
    } else {
        counters::REQUEST_SUCCESS_LABEL
    };
    let latency = start_time.elapsed();
    counters::MEMPOOL_SERVICE_LATENCY
        .with_label_values(&[counters::COMMIT_STATE_SYNC_LABEL, result])
        .observe(latency.as_secs_f64());
}

pub(crate) async fn process_consensus_request(mempool: &Mutex<CoreMempool>, req: ConsensusRequest) {
    //start latency timer
    let start_time = Instant::now();

    let (resp, callback, counter_label) = match req {
        ConsensusRequest::GetBlockRequest(max_block_size, transactions, callback) => {
            let exclude_transactions: HashSet<TxnPointer> = transactions
                .iter()
                .map(|txn| (txn.sender, txn.sequence_number))
                .collect();
            let mut txns;
            {
                let mut mempool = mempool.lock().expect("failed to acquire mempool lock");
                // gc before pulling block as extra protection against txns that may expire in consensus
                // Note: this gc operation relies on the fact that consensus uses the system time to determine block timestamp
                let curr_time = libra_time::duration_since_epoch();
                mempool.gc_by_expiration_time(curr_time);
                let block_size = cmp::max(max_block_size, 1);
                txns = mempool.get_block(block_size, exclude_transactions);
            }
            counters::MEMPOOL_SERVICE_TXNS
                .with_label_values(&[counters::GET_BLOCK_LABEL])
                .observe(txns.len() as f64);
            txns.len();
            let pulled_block = txns.drain(..).map(SignedTransaction::into).collect();

            (
                ConsensusResponse::GetBlockResponse(pulled_block),
                callback,
                counters::GET_BLOCK_LABEL,
            )
        }
        ConsensusRequest::RejectNotification(transactions, callback) => {
            // handle rejected txns
            counters::MEMPOOL_SERVICE_TXNS
                .with_label_values(&[counters::COMMIT_CONSENSUS_LABEL])
                .observe(transactions.len() as f64);
            commit_txns(mempool, transactions, 0, true).await;
            (
                ConsensusResponse::CommitResponse(),
                callback,
                counters::COMMIT_CONSENSUS_LABEL,
            )
        }
    };
    // send back to callback
    let result = if let Err(e) = callback.send(Ok(resp)).map_err(|_| {
        format_err!("[shared mempool] timeout on callback sending response to Mempool request")
    }) {
        error!(
            "[shared mempool] failed to send back mempool response with error: {:?}",
            e
        );
        counters::REQUEST_FAIL_LABEL
    } else {
        counters::REQUEST_SUCCESS_LABEL
    };
    let latency = start_time.elapsed();
    counters::MEMPOOL_SERVICE_LATENCY
        .with_label_values(&[counter_label, result])
        .observe(latency.as_secs_f64());
}

async fn commit_txns(
    mempool: &Mutex<CoreMempool>,
    transactions: Vec<CommittedTransaction>,
    block_timestamp_usecs: u64,
    is_rejected: bool,
) {
    let mut pool = mempool
        .lock()
        .expect("[shared mempool] failed to get mempool lock");

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

/// processes on-chain reconfiguration notification
pub(crate) async fn process_config_update<V>(
    config_update: OnChainConfigPayload,
    validator: Arc<RwLock<V>>,
) where
    V: TransactionValidation,
{
    // restart VM validator
    validator
        .write()
        .unwrap()
        .restart(config_update)
        .expect("failed to restart VM validator");
}
