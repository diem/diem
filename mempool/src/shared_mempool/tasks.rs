// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Tasks that are executed by coordinators (short-lived compared to coordinators)

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    counters,
    network::{MempoolNetworkSender, MempoolSyncMsg},
    shared_mempool::types::{
        notify_subscribers, PeerInfo, PeerSyncState, SharedMempool, SharedMempoolNotification,
    },
    CommitNotification, CommitResponse, CommittedTransaction, ConsensusRequest, ConsensusResponse,
    SubmissionStatus,
};
use anyhow::{ensure, format_err, Result};
use futures::{channel::oneshot, future::join_all};
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
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::RwLock;
use vm_validator::vm_validator::{get_account_sequence_number, TransactionValidation};

// ============================== //
//  broadcast_coordinator tasks  //
// ============================== //
/// sync routine
/// used to periodically broadcast ready to go transactions to peers
pub(crate) async fn sync_with_peers<'a>(
    peer_info: &'a Mutex<PeerInfo>,
    mempool: &'a Mutex<CoreMempool>,
    mut network_senders: HashMap<PeerId, MempoolNetworkSender>,
    batch_size: usize,
) {
    // Clone the underlying peer_info map and use this to sync and collect
    // state updates. We do this instead of holding the lock for the whole
    // function since that would hold the lock across await points which is bad.
    let peer_info_copy = peer_info
        .lock()
        .expect("[shared mempool] failed to acquire peer_info lock")
        .deref()
        .clone();
    let mut state_updates = vec![];

    for (peer_id, peer_state) in peer_info_copy.into_iter() {
        if peer_state.is_alive {
            let timeline_id = peer_state.timeline_id;
            let (transactions, new_timeline_id) = mempool
                .lock()
                .expect("[shared mempool] failed to acquire mempool lock")
                .read_timeline(timeline_id, batch_size);

            if !transactions.is_empty() {
                counters::SHARED_MEMPOOL_TRANSACTION_BROADCAST.inc_by(transactions.len() as i64);

                let network_sender = network_senders
                    .get_mut(&peer_state.network_id)
                    .expect("[shared mempool] missign network sender")
                    .clone();

                let request_id = create_request_id(timeline_id, new_timeline_id);
                if let Err(e) = send_mempool_sync_msg(
                    MempoolSyncMsg::BroadcastTransactionsRequest {
                        request_id,
                        transactions,
                    },
                    peer_id,
                    network_sender,
                ) {
                    error!(
                        "[shared mempool] error broadcasting transations to peer {}: {}",
                        peer_id, e
                    );
                } else {
                    // only update state for successful sends
                    state_updates.push((peer_id, new_timeline_id));
                }
            }
        }
    }

    // Lock the shared peer_info and apply state updates.
    let mut peer_info = peer_info
        .lock()
        .expect("[shared mempool] failed to acquire peer_info lock");
    for (peer_id, new_timeline_id) in state_updates {
        peer_info.entry(peer_id).and_modify(|t| {
            t.timeline_id = new_timeline_id;
        });
    }
}

fn send_mempool_sync_msg(
    msg: MempoolSyncMsg,
    recipient: PeerId,
    mut network_sender: MempoolNetworkSender,
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
        process_incoming_transactions(smp.clone(), vec![transaction], TimelineState::NotReady)
            .await;
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
    peer_id: PeerId,
    network_id: PeerId,
) where
    V: TransactionValidation,
{
    let network_sender = smp
        .network_senders
        .get_mut(&network_id)
        .expect("[shared mempool] missing network sender")
        .clone();
    let results = process_incoming_transactions(smp, transactions, timeline_state).await;
    log_txn_process_results(results, Some(peer_id));
    // send back ACK
    if let Err(e) = send_mempool_sync_msg(
        MempoolSyncMsg::BroadcastTransactionsResponse { request_id },
        peer_id,
        network_sender,
    ) {
        error!(
            "[shared mempool] failed to send ACK back to peer {}: {}",
            peer_id, e
        );
    }
}

/// submits a list of SignedTransaction to the local mempool
/// and returns a vector containing AdmissionControlStatus
async fn process_incoming_transactions<V>(
    smp: SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    timeline_state: TimelineState,
) -> Vec<SubmissionStatus>
where
    V: TransactionValidation,
{
    let mut statuses = vec![];

    let seq_numbers = join_all(
        transactions
            .iter()
            .map(|t| get_account_sequence_number(smp.storage_read_client.clone(), t.sender())),
    )
    .await;

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

    let validation_results = join_all(transactions.iter().map(|t| {
        let vm_validator = smp.validator.clone();
        async move {
            vm_validator
                .read()
                .await
                .validate_transaction(t.0.clone())
                .await
        }
    }))
    .await;

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
                        let mempool_status = mempool.add_txn(
                            transaction,
                            gas_amount,
                            rankin_score,
                            sequence_number,
                            timeline_state,
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
pub(crate) fn process_broadcast_ack<V>(
    smp: SharedMempool<V>,
    request_id: String,
    is_validator: bool, // whether this node is a validator or not
) where
    V: TransactionValidation,
{
    if is_validator {
        return;
    }

    match parse_request_id(request_id) {
        Ok((start_id, end_id)) => {
            let mut mempool = smp
                .mempool
                .lock()
                .expect("[shared mempool] failed to acquire mempool lock");

            for txn in mempool.timeline_range(start_id, end_id).iter() {
                mempool.remove_transaction(&txn.sender(), txn.sequence_number(), false);
            }
        }
        Err(err) => warn!("[shared mempool] ACK with invalid request_id: {:?}", err),
    }
}

// ===================== //
// peer management tasks //
// ===================== //
/// new peer discovery handler
/// adds new entry to `peer_info`
/// `network_id` is the ID of the mempool network the peer belongs to
pub(crate) fn new_peer(peer_info: &Mutex<PeerInfo>, peer_id: PeerId, network_id: PeerId) {
    peer_info
        .lock()
        .expect("[shared mempool] failed to acquire peer_info lock")
        .entry(peer_id)
        .or_insert(PeerSyncState {
            timeline_id: 0,
            is_alive: true,
            network_id,
        })
        .is_alive = true;
}

/// lost peer handler. Marks connection as dead
pub(crate) fn lost_peer(peer_info: &Mutex<PeerInfo>, peer_id: PeerId) {
    if let Some(state) = peer_info
        .lock()
        .expect("[shared mempool] failed to acquire peer_info lock")
        .get_mut(&peer_id)
    {
        state.is_alive = false;
    }
}

// ================================= //
// intra-node communication handlers //
// ================================= //
pub(crate) async fn process_state_sync_request<V>(smp: SharedMempool<V>, req: CommitNotification)
where
    V: TransactionValidation,
{
    commit_txns(smp, req.transactions, req.block_timestamp_usecs, false).await;
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

pub(crate) async fn process_consensus_request<V>(smp: SharedMempool<V>, req: ConsensusRequest)
where
    V: TransactionValidation,
{
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
            let mut txns = smp
                .mempool
                .lock()
                .expect("[get_block] acquire mempool lock")
                .get_block(block_size, exclude_transactions);
            let transactions = txns.drain(..).map(SignedTransaction::into).collect();

            (ConsensusResponse::GetBlockResponse(transactions), callback)
        }
        ConsensusRequest::RejectNotification(transactions, callback) => {
            // handle rejected txns
            commit_txns(smp, transactions, 0, true).await;
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

async fn commit_txns<V>(
    smp: SharedMempool<V>,
    transactions: Vec<CommittedTransaction>,
    block_timestamp_usecs: u64,
    is_rejected: bool,
) where
    V: TransactionValidation,
{
    let mut pool = smp
        .mempool
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
        .await
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
