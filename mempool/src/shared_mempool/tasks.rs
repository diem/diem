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
use anyhow::{ensure, format_err, Result};
use futures::{channel::oneshot, stream::FuturesUnordered};
use libra_config::config::PeerNetworkId;
use libra_logger::prelude::*;
use libra_types::{
    mempool_status::{MempoolStatus, MempoolStatusCode},
    on_chain_config::OnChainConfigPayload,
    transaction::SignedTransaction,
    vm_error::{
        StatusCode::{RESOURCE_DOES_NOT_EXIST, SEQUENCE_NUMBER_TOO_OLD},
        VMStatus,
    },
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
    smp: &mut SharedMempool<V>,
    scheduled_broadcasts: &mut FuturesUnordered<ScheduledBroadcast>,
    executor: Handle,
) where
    V: TransactionValidation,
{
    broadcast_single_peer(peer, smp);

    let interval_ms = smp.config.shared_mempool_tick_interval_ms;
    scheduled_broadcasts.push(ScheduledBroadcast::new(
        Instant::now() + Duration::from_millis(interval_ms),
        peer,
        executor,
    ))
}

/// broadcasts txns to `peer` if alive
fn broadcast_single_peer<V>(peer: PeerNetworkId, smp: &mut SharedMempool<V>)
where
    V: TransactionValidation,
{
    let peer_manager = &smp.peer_manager;
    let timeline_id = if peer_manager.is_picked_peer(peer) {
        let state = peer_manager.get_peer_state(peer);
        if state.is_alive {
            state.timeline_id
        } else {
            return;
        }
    } else {
        return;
    };

    let (transactions, new_timeline_id) = smp
        .mempool
        .lock()
        .expect("[shared mempool] failed to acquire mempool lock")
        .read_timeline(timeline_id, smp.config.shared_mempool_batch_size);

    if transactions.is_empty() {
        return;
    }

    let mut network_sender = smp
        .network_senders
        .get_mut(&peer.network_id())
        .expect("[shared mempool] missing network sender");

    debug!("[mempool] sending batch {}:{}", timeline_id, new_timeline_id);
    let request_id = create_request_id(timeline_id, new_timeline_id);
    let txns_ct = transactions.len();
    if let Err(e) = send_mempool_sync_msg(
        MempoolSyncMsg::BroadcastTransactionsRequest {
            request_id,
            transactions,
        },
        peer.peer_id(),
        &mut network_sender,
    ) {
        error!(
            "[shared mempool] error broadcasting transactions to peer {:?}: {}",
            peer, e
        );
    } else {
        counters::SHARED_MEMPOOL_TRANSACTION_BROADCAST.inc_by(txns_ct as i64);
        peer_manager.update_peer_broadcast(peer, new_timeline_id);
        notify_subscribers(SharedMempoolNotification::Broadcast, &smp.subscribers);
    }
}

fn send_mempool_sync_msg(
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
    let mut statuses =
        process_incoming_transactions(&smp, vec![transaction], TimelineState::NotReady).await;
    log_txn_process_results(statuses.clone(), None);
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
    let results = process_incoming_transactions(&smp, transactions, timeline_state).await;
    log_txn_process_results(results, Some(peer.peer_id()));
    // send back ACK
    let mut network_sender = smp
        .network_senders
        .get_mut(&peer.network_id())
        .expect("[shared mempool] missing network sender");
    if let Err(e) = send_mempool_sync_msg(
        MempoolSyncMsg::BroadcastTransactionsResponse { request_id },
        peer.peer_id(),
        &mut network_sender,
    ) {
        error!(
            "[shared mempool] failed to send ACK back to peer {:?}: {}",
            peer, e
        );
    }
}

/// submits a list of SignedTransaction to the local mempool
/// and returns a vector containing AdmissionControlStatus
async fn process_incoming_transactions<V>(
    smp: &SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    timeline_state: TimelineState,
) -> Vec<SubmissionStatus>
where
    V: TransactionValidation,
{
    let mut statuses = vec![];

    let seq_numbers = transactions
        .iter()
        .map(|t| get_account_sequence_number(smp.db.as_ref(), t.sender()))
        .collect::<Vec<_>>();

    let transactions: Vec<_> =
        transactions
            .into_iter()
            .enumerate()
            .filter_map(|(idx, t)| {
                if let Ok(sequence_number) = seq_numbers[idx] {
                    if t.sequence_number() >= sequence_number {
                        return Some((t, sequence_number));
                    } else {
                        statuses.push((
                            MempoolStatus::new(MempoolStatusCode::VmError),
                            Some(VMStatus::new(SEQUENCE_NUMBER_TOO_OLD)),
                        ));
                    }
                } else {
                    // failed to get transaction
                    statuses.push((
                        MempoolStatus::new(MempoolStatusCode::VmError),
                        Some(VMStatus::new(RESOURCE_DOES_NOT_EXIST).with_message(
                            "[shared mempool] failed to get account state".to_string(),
                        )),
                    ));
                }
                None
            })
            .collect();

    let validation_results = transactions
        .iter()
        .map(|t| {
            smp.validator
                .read()
                .unwrap()
                .validate_transaction(t.0.clone())
        })
        .collect::<Vec<_>>();

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
                        let rankin_score = validation_result.score();
                        let is_governance_txn = validation_result.is_governance_txn();
                        let mempool_status = mempool.add_txn(
                            transaction,
                            gas_amount,
                            rankin_score,
                            sequence_number,
                            timeline_state,
                            is_governance_txn,
                        );
                        statuses.push((mempool_status, None));
                    }
                    Some(validation_status) => {
                        statuses.push((
                            MempoolStatus::new(MempoolStatusCode::VmError),
                            Some(validation_status.clone()),
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
fn log_txn_process_results(results: Vec<SubmissionStatus>, sender: Option<PeerId>) {
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

/// processes ACK from peer node regarding txn submission to that node
pub(crate) fn process_broadcast_ack(
    mempool: &Mutex<CoreMempool>,
    request_id: String,
    is_validator: bool, // whether this node is a validator or not
) {
    if is_validator {
        return;
    }

    match parse_request_id(request_id) {
        Ok((start_id, end_id)) => {
            debug!("[mempool] received ACK {}:{}", start_id, end_id);
            let mut mempool = mempool
                .lock()
                .expect("[shared mempool] failed to acquire mempool lock");

            for txn in mempool.timeline_range(start_id, end_id).iter() {
                mempool.remove_transaction(&txn.sender(), txn.sequence_number(), false);
            }
        }
        Err(err) => warn!("[shared mempool] ACK with invalid request_id: {:?}", err),
    }
}

// ================================= //
// intra-node communication handlers //
// ================================= //
pub(crate) async fn process_state_sync_request(
    mempool: Arc<Mutex<CoreMempool>>,
    req: CommitNotification,
) {
    commit_txns(&mempool, req.transactions, req.block_timestamp_usecs, false).await;
    // send back to callback
    if let Err(e) = req
        .callback
        .send(Ok(CommitResponse {
            msg: "".to_string(),
        }))
        .map_err(|_| {
            format_err!("[shared mempool] timeout on callback sending response to Mempool request")
        })
    {
        error!(
            "[shared mempool] failed to send back CommitResponse with error: {:?}",
            e
        );
    }
}

pub(crate) async fn process_consensus_request(mempool: &Mutex<CoreMempool>, req: ConsensusRequest) {
    let (resp, callback) = match req {
        ConsensusRequest::GetBlockRequest(max_block_size, transactions, callback) => {
            let block_size = cmp::max(max_block_size, 1);
            counters::MEMPOOL_SERVICE
                .with_label_values(&["get_block", "requested"])
                .inc_by(block_size as i64);

            let exclude_transactions: HashSet<TxnPointer> = transactions
                .iter()
                .map(|txn| (txn.sender, txn.sequence_number))
                .collect();
            let mut txns = mempool
                .lock()
                .expect("[get_block] acquire mempool lock")
                .get_block(block_size, exclude_transactions);
            let transactions = txns.drain(..).map(SignedTransaction::into).collect();

            (ConsensusResponse::GetBlockResponse(transactions), callback)
        }
        ConsensusRequest::RejectNotification(transactions, callback) => {
            // handle rejected txns
            commit_txns(mempool, transactions, 0, true).await;
            (ConsensusResponse::CommitResponse(), callback)
        }
    };
    // send back to callback
    if let Err(e) = callback.send(Ok(resp)).map_err(|_| {
        format_err!("[shared mempool] timeout on callback sending response to Mempool request")
    }) {
        error!(
            "[shared mempool] failed to send back mempool response with error: {:?}",
            e
        );
    }
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

/// creates uniques request id for the batch in the format "{start_id}_{end_id}"
/// where start is an id in timeline index  that is lower than the first txn in a batch
/// and end equals to timeline ID of last transaction in a batch
fn create_request_id(start_timeline_id: u64, end_timeline_id: u64) -> String {
    format!("{}_{}", start_timeline_id, end_timeline_id)
}

/// parses request_id according to format "{start_id}_{end_id}"
fn parse_request_id(request_id: String) -> Result<(u64, u64)> {
    let timeline_ids: Vec<_> = request_id.split('_').collect();
    ensure!(timeline_ids.len() == 2, "invalid request_id {}", request_id);
    let start_id = timeline_ids[0].parse::<u64>()?;
    let end_id = timeline_ids[1].parse::<u64>()?;
    ensure!(start_id < end_id, "invalid broadcast range {}", request_id);
    Ok((start_id, end_id))
}
