// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    counters,
    network::{MempoolNetworkEvents, MempoolNetworkSender, MempoolSyncMsg},
};
use anyhow::{format_err, Result};
use bounded_executor::BoundedExecutor;
use futures::{
    channel::{
        mpsc::{self, Receiver, UnboundedSender},
        oneshot,
    },
    future::join_all,
    stream::select_all,
    Stream, StreamExt,
};
use libra_config::config::{MempoolConfig, NodeConfig};
use libra_logger::prelude::*;
use libra_security_logger::{security_log, SecurityEvent};
use libra_types::{
    account_address::AccountAddress,
    mempool_status::{MempoolStatus, MempoolStatusCode},
    transaction::SignedTransaction,
    vm_error::{
        StatusCode::{RESOURCE_DOES_NOT_EXIST, SEQUENCE_NUMBER_TOO_OLD},
        VMStatus,
    },
    PeerId,
};
use network::protocols::network::Event;
use std::{
    cmp,
    collections::{HashMap, HashSet},
    ops::Deref,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};
use storage_client::{StorageRead, StorageReadServiceClient};
use tokio::{
    runtime::{Builder, Handle, Runtime},
    time::interval,
};
use vm_validator::vm_validator::{get_account_state, TransactionValidation, VMValidator};

/// state of last sync with peer
/// `timeline_id` is position in log of ready transactions
/// `is_alive` - is connection healthy
/// `network_id` - ID of the mempool network that this peer belongs to
#[derive(Clone)]
struct PeerSyncState {
    timeline_id: u64,
    is_alive: bool,
    network_id: PeerId,
}

/// stores only peers that receive txns from this node
type PeerInfo = HashMap<PeerId, PeerSyncState>;

/// Outbound peer syncing event emitted by [`IntervalStream`].
#[derive(Debug)]
pub(crate) struct SyncEvent;

type IntervalStream = Pin<Box<dyn Stream<Item = SyncEvent> + Send + 'static>>;

/// Submission Status is represented as combination of vm_validator internal status and core mempool insertion status
pub type SubmissionStatus = (MempoolStatus, Option<VMStatus>);

/// sender type: used to enqueue new transactions to shared mempool by client endpoints
pub type MempoolClientSender =
    mpsc::Sender<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SharedMempoolNotification {
    Sync,
    PeerStateChange,
    NewTransactions,
    ACK,
}

/// Struct that owns all dependencies required by shared mempool routines
#[derive(Clone)]
struct SharedMempool<V>
where
    V: TransactionValidation + 'static,
{
    mempool: Arc<Mutex<CoreMempool>>,
    config: MempoolConfig,
    network_senders: HashMap<PeerId, MempoolNetworkSender>,
    storage_read_client: Arc<dyn StorageRead>,
    validator: Arc<V>,
    peer_info: Arc<Mutex<PeerInfo>>,
    subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
}

/// Message sent from consensus to mempool
pub enum ConsensusRequest {
    /// request to pull block to submit to consensus
    GetBlockRequest(
        // max block size
        u64,
        // transactions to exclude from requested block
        Vec<TransactionExclusion>,
        // callback to send response back to sender
        oneshot::Sender<Result<ConsensusResponse>>,
    ),
    /// notifications about *rejected* committed txns
    RejectNotification(
        // committed transactions
        Vec<CommittedTransaction>,
        // callback to send response back to sender
        oneshot::Sender<Result<ConsensusResponse>>,
    ),
}

/// Response setn from mempool to consensus
pub enum ConsensusResponse {
    /// block to submit to consensus
    GetBlockResponse(
        // transactions in block
        Vec<SignedTransaction>,
    ),
    /// ACK for commit notification
    CommitResponse(),
}

/// notification from state sync to mempool of commit event
/// This notifies mempool to remove committed txns
pub struct CommitNotification {
    /// committed transactions
    pub transactions: Vec<CommittedTransaction>,
    /// timestamp of committed block
    pub block_timestamp_usecs: u64,
    /// callback to send back response from mempool to State Sync
    pub callback: oneshot::Sender<Result<CommitResponse>>,
}

/// ACK response to commit notification
#[derive(Debug)]
pub struct CommitResponse {
    /// error msg if applicable - empty string if commit was processed successfully by mempool
    pub msg: String,
}

/// successfully executed and committed txn
pub struct CommittedTransaction {
    /// sender
    pub sender: AccountAddress,
    /// sequence number
    pub sequence_number: u64,
}

/// excluded txn
pub struct TransactionExclusion {
    /// sender
    pub sender: AccountAddress,
    /// sequence number
    pub sequence_number: u64,
}

fn notify_subscribers(
    event: SharedMempoolNotification,
    subscribers: &[UnboundedSender<SharedMempoolNotification>],
) {
    for subscriber in subscribers {
        let _ = subscriber.unbounded_send(event);
    }
}

fn default_timer(tick_ms: u64) -> IntervalStream {
    interval(Duration::from_millis(tick_ms))
        .map(|_| SyncEvent)
        .boxed()
}

/// new peer discovery handler
/// adds new entry to `peer_info`
/// `network_id` is the ID of the mempool network the peer belongs to
fn new_peer(peer_info: &Mutex<PeerInfo>, peer_id: PeerId, network_id: PeerId) {
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
fn lost_peer(peer_info: &Mutex<PeerInfo>, peer_id: PeerId) {
    if let Some(state) = peer_info
        .lock()
        .expect("[shared mempool] failed to acquire peer_info lock")
        .get_mut(&peer_id)
    {
        state.is_alive = false;
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

/// sync routine
/// used to periodically broadcast ready to go transactions to peers
async fn sync_with_peers<'a>(
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
                if let Err(e) = send_mempool_sync_msg(
                    MempoolSyncMsg::BroadcastTransactionsRequest(
                        (timeline_id, new_timeline_id),
                        transactions,
                    ),
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

    let account_states = join_all(
        transactions
            .iter()
            .map(|t| get_account_state(smp.storage_read_client.clone(), t.sender())),
    )
    .await;

    let transactions: Vec<_> =
        transactions
            .into_iter()
            .enumerate()
            .filter_map(|(idx, t)| {
                if let Ok((sequence_number, balance)) = account_states[idx] {
                    if t.sequence_number() >= sequence_number {
                        return Some((t, sequence_number, balance));
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

    let validations = join_all(
        transactions
            .iter()
            .map(|t| smp.validator.validate_transaction(t.0.clone())),
    )
    .await;

    {
        let mut mempool = smp
            .mempool
            .lock()
            .expect("[shared mempool] failed to acquire mempool lock");
        for (idx, (transaction, sequence_number, balance)) in transactions.into_iter().enumerate() {
            if let Ok(None) = validations[idx] {
                let gas_cost = transaction.max_gas_amount();

                let mempool_status = mempool.add_txn(
                    transaction,
                    gas_cost,
                    sequence_number,
                    balance,
                    timeline_state,
                );
                statuses.push((mempool_status, None));
            } else if let Ok(Some(validation_status)) = &validations[idx] {
                statuses.push((
                    MempoolStatus::new(MempoolStatusCode::VmError),
                    Some(validation_status.clone()),
                ));
            }
        }
    }
    notify_subscribers(SharedMempoolNotification::NewTransactions, &smp.subscribers);
    statuses
}

async fn process_client_transaction_submission<V>(
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

async fn process_transaction_broadcast<V>(
    mut smp: SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    start_id: u64,
    end_id: u64,
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
        MempoolSyncMsg::BroadcastTransactionsResponse(start_id, end_id),
        peer_id,
        network_sender,
    ) {
        error!(
            "[shared mempool] failed to send ACK back to peer {}: {}",
            peer_id, e
        );
    }
}

fn process_broadcast_ack<V>(smp: SharedMempool<V>, start_id: u64, end_id: u64, is_validator: bool)
where
    V: TransactionValidation,
{
    if is_validator {
        return;
    }
    if start_id < end_id {
        let mut mempool = smp
            .mempool
            .lock()
            .expect("[shared mempool] failed to acquire mempool lock");

        for txn in mempool.timeline_range(start_id, end_id).iter() {
            mempool.remove_transaction(&txn.sender(), txn.sequence_number(), false);
        }
    } else {
        warn!(
            "[shared mempool] ACK with invalid broadcast range {} to {}",
            start_id, end_id
        );
    }
}

/// This task handles [`SyncEvent`], which is periodically emitted for us to
/// broadcast ready to go transactions to peers.
async fn outbound_sync_task<V>(smp: SharedMempool<V>, mut interval: IntervalStream)
where
    V: TransactionValidation,
{
    let peer_info = smp.peer_info;
    let mempool = smp.mempool;
    let network_senders = smp.network_senders;
    let batch_size = smp.config.shared_mempool_batch_size;
    let subscribers = smp.subscribers;

    while let Some(sync_event) = interval.next().await {
        trace!("SyncEvent: {:?}", sync_event);
        sync_with_peers(&peer_info, &mempool, network_senders.clone(), batch_size).await;
        notify_subscribers(SharedMempoolNotification::Sync, &subscribers);
    }

    crit!("SharedMempool outbound_sync_task terminated");
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

async fn process_state_sync_request<V>(smp: SharedMempool<V>, req: CommitNotification)
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

async fn process_consensus_request<V>(smp: SharedMempool<V>, req: ConsensusRequest)
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

/// This task handles inbound network events.
async fn inbound_network_task<V>(
    smp: SharedMempool<V>,
    executor: Handle,
    network_events: Vec<(PeerId, MempoolNetworkEvents)>,
    mut client_events: mpsc::Receiver<(
        SignedTransaction,
        oneshot::Sender<Result<SubmissionStatus>>,
    )>,
    mut consensus_requests: mpsc::Receiver<ConsensusRequest>,
    mut state_sync_requests: mpsc::Receiver<CommitNotification>,
    node_config: NodeConfig,
) where
    V: TransactionValidation,
{
    let peer_info = smp.peer_info.clone();
    let subscribers = smp.subscribers.clone();
    let smp_events: Vec<_> = network_events
        .into_iter()
        .map(|(network_id, events)| events.map(move |e| (network_id, e)))
        .collect();
    let mut events = select_all(smp_events).fuse();
    let is_validator = node_config.base.role.is_validator();

    // Use a BoundedExecutor to restrict only `workers_available` concurrent
    // worker tasks that can process incoming transactions.
    let workers_available = smp.config.shared_mempool_max_concurrent_inbound_syncs;
    let bounded_executor = BoundedExecutor::new(workers_available, executor);

    loop {
        ::futures::select! {
            (mut msg, callback) = client_events.select_next_some() => {
                bounded_executor
                .spawn(process_client_transaction_submission(
                    smp.clone(),
                    msg,
                    callback,
                ))
                .await;
            },
            msg = consensus_requests.select_next_some() => {
                process_consensus_request(smp.clone(), msg).await;
            }
            msg = state_sync_requests.select_next_some() => {
                tokio::spawn(process_state_sync_request(smp.clone(), msg));
            },
            (network_id, event) = events.select_next_some() => {
                match event {
                    Ok(network_event) => {
                        match network_event {
                            Event::NewPeer(peer_id) => {
                                counters::SHARED_MEMPOOL_EVENTS
                                    .with_label_values(&["new_peer".to_string().deref()])
                                    .inc();
                                if node_config.is_upstream_peer(peer_id, network_id) {
                                    new_peer(&peer_info, peer_id, network_id);
                                }
                                notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                            }
                            Event::LostPeer(peer_id) => {
                                counters::SHARED_MEMPOOL_EVENTS
                                    .with_label_values(&["lost_peer".to_string().deref()])
                                    .inc();
                                if node_config.is_upstream_peer(peer_id, network_id) {
                                    lost_peer(&peer_info, peer_id);
                                }
                                notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                            }
                            Event::Message((peer_id, msg)) => {
                                counters::SHARED_MEMPOOL_EVENTS
                                    .with_label_values(&["message".to_string().deref()])
                                    .inc();
                                match msg {
                                    MempoolSyncMsg::BroadcastTransactionsRequest((start_id, end_id), transactions) => {
                                        counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                                            .with_label_values(&["received".to_string().deref(), peer_id.to_string().deref()])
                                            .inc_by(transactions.len() as i64);
                                        let smp_clone = smp.clone();
                                        let timeline_state = match node_config.is_upstream_peer(peer_id, network_id) {
                                            true => TimelineState::NonQualified,
                                            false => TimelineState::NotReady,
                                        };
                                        bounded_executor
                                            .spawn(process_transaction_broadcast(
                                                smp_clone,
                                                transactions,
                                                start_id,
                                                end_id,
                                                timeline_state,
                                                peer_id,
                                                network_id,
                                            ))
                                            .await;
                                    }
                                    MempoolSyncMsg::BroadcastTransactionsResponse(start_id, end_id) => {
                                        process_broadcast_ack(smp.clone(), start_id, end_id, is_validator);
                                        notify_subscribers(SharedMempoolNotification::ACK, &smp.subscribers);
                                    }
                                };
                            }
                            _ => {
                                security_log(SecurityEvent::InvalidNetworkEventMP)
                                    .error("UnexpectedNetworkEvent")
                                    .data(&network_event)
                                    .log();
                                debug_assert!(false, "Unexpected network event");
                            }
                        }
                    },
                    Err(e) => {
                        security_log(SecurityEvent::InvalidNetworkEventMP)
                            .error(&e)
                            .log();
                    }
                };
            },
            complete => break,
        }
    }
    crit!("[shared mempool] inbound_network_task terminated");
}

/// GC all expired transactions by SystemTTL
async fn gc_task(mempool: Arc<Mutex<CoreMempool>>, gc_interval_ms: u64) {
    let mut interval = interval(Duration::from_millis(gc_interval_ms));
    while let Some(_interval) = interval.next().await {
        mempool
            .lock()
            .expect("[shared mempool] failed to acquire mempool lock")
            .gc_by_system_ttl();
    }

    crit!("SharedMempool gc_task terminated");
}

/// bootstrap of SharedMempool
/// creates separate Tokio Runtime that runs following routines:
///   - outbound_sync_task (task that periodically broadcasts transactions to peers)
///   - inbound_network_task (task that handles inbound mempool messages and network events)
///   - gc_task (task that performs GC of all expired transactions by SystemTTL)
pub(crate) fn start_shared_mempool<V>(
    executor: &Handle,
    config: &NodeConfig,
    mempool: Arc<Mutex<CoreMempool>>,
    // First element in tuple is the network ID
    // See `NodeConfig::is_upstream_peer` for the definition of network ID
    mempool_network_handles: Vec<(PeerId, MempoolNetworkSender, MempoolNetworkEvents)>,
    client_events: mpsc::Receiver<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>,
    consensus_requests: mpsc::Receiver<ConsensusRequest>,
    state_sync_requests: mpsc::Receiver<CommitNotification>,
    storage_read_client: Arc<dyn StorageRead>,
    validator: Arc<V>,
    subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
    timer: Option<IntervalStream>,
) where
    V: TransactionValidation + 'static,
{
    let peer_info = Arc::new(Mutex::new(PeerInfo::new()));
    let config_clone = config.clone_for_template();

    let mut all_network_events = vec![];
    let mut network_senders = HashMap::new();
    for (network_id, network_sender, network_events) in mempool_network_handles.into_iter() {
        all_network_events.push((network_id, network_events));
        network_senders.insert(network_id, network_sender);
    }

    let smp = SharedMempool {
        mempool: mempool.clone(),
        config: config.mempool.clone(),
        network_senders,
        storage_read_client,
        validator,
        peer_info,
        subscribers,
    };

    let interval_ms = config.mempool.shared_mempool_tick_interval_ms;
    let smp_outbound = smp.clone();
    let f = async move {
        let interval = timer.unwrap_or_else(|| default_timer(interval_ms));
        outbound_sync_task(smp_outbound, interval).await
    };

    executor.spawn(f);

    executor.spawn(inbound_network_task(
        smp,
        executor.clone(),
        all_network_events,
        client_events,
        consensus_requests,
        state_sync_requests,
        config_clone,
    ));

    executor.spawn(gc_task(
        mempool,
        config.mempool.system_transaction_gc_interval_ms,
    ));
}

/// method used to bootstrap shared mempool for a node
pub fn bootstrap(
    config: &NodeConfig,
    // The first element in the tuple is the ID of the network that this network is a handle to
    // See `NodeConfig::is_upstream_peer` for the definition of network ID
    mempool_network_handles: Vec<(PeerId, MempoolNetworkSender, MempoolNetworkEvents)>,
    client_events: Receiver<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>,
    consensus_requests: Receiver<ConsensusRequest>,
    state_sync_requests: Receiver<CommitNotification>,
) -> Runtime {
    let runtime = Builder::new()
        .thread_name("shared-mem-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[shared mempool] failed to create runtime");
    let executor = runtime.handle();
    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
    let storage_client: Arc<dyn StorageRead> =
        Arc::new(StorageReadServiceClient::new(&config.storage.address));
    let vm_validator = Arc::new(VMValidator::new(
        &config,
        Arc::clone(&storage_client),
        executor.clone(),
    ));
    start_shared_mempool(
        runtime.handle(),
        config,
        mempool,
        mempool_network_handles,
        client_events,
        consensus_requests,
        state_sync_requests,
        storage_client,
        vm_validator,
        vec![],
        None,
    );

    runtime
}
