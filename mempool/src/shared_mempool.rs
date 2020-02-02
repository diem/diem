// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState, TxnPointer},
    counters,
};
use admission_control_proto::{
    proto::admission_control::{
        submit_transaction_response::Status, AdmissionControlStatusCode, SubmitTransactionRequest,
        SubmitTransactionResponse,
    },
    AdmissionControlStatus,
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
use libra_mempool_shared_proto::proto::mempool_status::{
    MempoolAddTransactionStatus as MempoolAddTransactionStatusProto,
    MempoolAddTransactionStatusCode,
};
use libra_types::{
    account_address::AccountAddress,
    proto::types::{SignedTransaction as SignedTransactionProto, VmStatus as VmStatusProto},
    transaction::SignedTransaction,
    vm_error::{StatusCode::RESOURCE_DOES_NOT_EXIST, VMStatus},
    PeerId,
};
use network::{
    proto::MempoolSyncMsg,
    validator_network::{Event, MempoolNetworkEvents, MempoolNetworkSender},
};
use std::{
    cmp,
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    ops::Deref,
    pin::Pin,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SharedMempoolNotification {
    Sync,
    PeerStateChange,
    NewTransactions,
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

/// request to get block, sent by consensus to mempool
pub struct GetBlockRequest {
    /// max block size
    pub max_block_size: u64,
    /// transactions to exclude from this block
    pub transactions: Vec<TransactionExclusion>,
    /// callback to send response to sender
    pub callback: oneshot::Sender<Result<GetBlockResponse>>,
}

/// get block response from mempool to consensus
pub struct GetBlockResponse {
    /// transactions in the requested block
    pub transactions: Vec<SignedTransaction>,
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
pub struct CommitResponse {}

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

                let mut msg = MempoolSyncMsg::default();
                msg.peer_id = peer_id.into();
                msg.transactions = transactions
                    .into_iter()
                    .map(|txn| txn.try_into().unwrap())
                    .collect();

                // Since this is a direct-send, this will only error if the network
                // module has unexpectedly crashed or shutdown.
                let network_sender = network_senders.get_mut(&peer_state.network_id).unwrap();
                network_sender
                    .clone()
                    .send_to(peer_id, msg)
                    .await
                    .expect("[shared mempool] failed to direct-send mempool sync message");
            }

            state_updates.push((peer_id, new_timeline_id));
        }
    }

    // Lock the shared peer_info and apply state updates.
    let mut peer_info = peer_info
        .lock()
        .expect("[shared mempool] failed to acquire peer_info lock");
    for (peer_id, new_timeline_id) in state_updates {
        peer_info
            .entry(peer_id)
            .and_modify(|t| t.timeline_id = new_timeline_id);
    }
}

fn convert_txn_from_proto(txn_proto: SignedTransactionProto) -> Option<SignedTransaction> {
    match SignedTransaction::try_from(txn_proto.clone()) {
        Ok(txn) => Some(txn),
        Err(e) => {
            security_log(SecurityEvent::InvalidTransactionMP)
                .error(&e)
                .data(&txn_proto)
                .log();
            None
        }
    }
}

/// submits a list of SignedTransaction to the local mempool
/// and returns a vector containing AdmissionControlStatus
async fn process_incoming_transactions<V>(
    smp: SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    timeline_state: TimelineState,
) -> Vec<Status>
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

    // eagerly filter out transactions that were already committed
    let transactions: Vec<_> = transactions
        .into_iter()
        .enumerate()
        .filter_map(|(idx, t)| {
            if let Ok((sequence_number, balance)) = account_states[idx] {
                if t.sequence_number() >= sequence_number {
                    return Some((t, sequence_number, balance));
                }
            } else {
                // failed to get transaction
                statuses.push(Status::VmStatus(VmStatusProto::from(
                    VMStatus::new(RESOURCE_DOES_NOT_EXIST)
                        .with_message("[shared mempool] failed to get account state".to_string()),
                )));
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

                if mempool_status.code == MempoolAddTransactionStatusCode::Valid {
                    statuses.push(Status::AcStatus(AdmissionControlStatus::Accepted.into()));
                } else {
                    statuses.push(Status::MempoolStatus(
                        MempoolAddTransactionStatusProto::from(mempool_status),
                    ));
                }
            } else if let Ok(Some(validation_status)) = &validations[idx] {
                statuses.push(Status::VmStatus(VmStatusProto::from(
                    validation_status.clone(),
                )));
            }
        }
    }
    notify_subscribers(SharedMempoolNotification::NewTransactions, &smp.subscribers);
    statuses
}

async fn process_client_transaction_submission<V>(
    smp: SharedMempool<V>,
    req: SubmitTransactionRequest,
    callback: oneshot::Sender<Result<SubmitTransactionResponse>>,
) where
    V: TransactionValidation,
{
    let mut response = SubmitTransactionResponse::default();
    let txn_proto = req
        .transaction
        .clone()
        .unwrap_or_else(SignedTransactionProto::default);

    // get status from attempt to submit txns
    match convert_txn_from_proto(txn_proto) {
        None => {
            response.status = Some(Status::AcStatus(
                AdmissionControlStatus::Rejected("submit txn rejected".to_string()).into(),
            ));
        }
        Some(txn) => {
            let mut statuses =
                process_incoming_transactions(smp.clone(), vec![txn], TimelineState::NotReady)
                    .await;
            log_txn_process_results(statuses.clone(), None);
            if statuses.is_empty() {
                error!("[shared mempool] unexpected error happened");
            } else {
                response.status = Some(statuses.remove(0));
            }
        }
    }

    if let Err(e) = callback
        .send(Ok(response))
        .map_err(|_| format_err!("[shared mempool] timeout on callback send to AC endpoint"))
    {
        error!("[shared mempool] failed to send back transaction submission result to AC endpoint with error: {:?}", e);
    }
}

fn log_txn_process_results(results: Vec<Status>, sender: Option<PeerId>) {
    let sender = match sender {
        Some(peer) => peer.to_string(),
        None => "client".to_string(),
    };
    for result in results.iter() {
        match result {
            Status::AcStatus(ac_status) => {
                // log success
                if ac_status.code() == AdmissionControlStatusCode::Accepted {
                    counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                        .with_label_values(&["success".to_string().deref(), &sender])
                        .inc();
                }
            }
            Status::VmStatus(_) => {
                // log vm validation failure
                counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                    .with_label_values(&["validation_failed".to_string().deref(), &sender])
                    .inc();
            }
            Status::MempoolStatus(mempool_status_proto) => {
                // log mempool status failure
                counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                    .with_label_values(&[
                        format!("{:?}", mempool_status_proto.code).deref(),
                        &sender,
                    ])
                    .inc();
            }
        }
    }
}

async fn process_transaction_broadcast<V>(
    smp: SharedMempool<V>,
    transactions: Vec<SignedTransaction>,
    timeline_state: TimelineState,
    peer_id: PeerId,
) where
    V: TransactionValidation,
{
    let results = process_incoming_transactions(smp, transactions, timeline_state).await;
    log_txn_process_results(results, Some(peer_id));
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

async fn process_state_sync_request<V>(smp: SharedMempool<V>, req: CommitNotification)
where
    V: TransactionValidation,
{
    {
        let mut pool = smp
            .mempool
            .lock()
            .expect("[shared mempool] failed to get mempool lock");
        if !req.transactions.is_empty() {
            debug!("[shared mempool] state sync req removing {:?} txns", req.transactions.len());
        }
        for transaction in req.transactions {
            pool.remove_transaction(&transaction.sender, transaction.sequence_number, false);
        }
        if req.block_timestamp_usecs > 0 {
            pool.gc_by_expiration_time(Duration::from_micros(req.block_timestamp_usecs));
        }
    }

    let resp = CommitResponse {};
    // send back to callback
    if let Err(e) = req.callback.send(Ok(resp)).map_err(|_| {
        format_err!("[shared mempool] timeout on callback sending response to Mempool request")
    }) {
        error!(
            "[shared mempool] failed to send back CommitResponse with error: {:?}",
            e
        );
    }
}

async fn process_consensus_request<V>(smp: SharedMempool<V>, req: GetBlockRequest)
where
    V: TransactionValidation,
{
    let block_size = cmp::max(req.max_block_size, 1);
    counters::MEMPOOL_SERVICE
        .with_label_values(&["get_block", "requested"])
        .inc_by(block_size as i64);

    let exclude_transactions: HashSet<TxnPointer> = req
        .transactions
        .iter()
        .map(|txn| (txn.sender, txn.sequence_number))
        .collect();

    let mut txns = smp
        .mempool
        .lock()
        .expect("[get_block] acquire mempool lock")
        .get_block(block_size, exclude_transactions);

    let transactions: Vec<SignedTransaction> = txns.drain(..).map(SignedTransaction::into).collect();

    debug!("[shared mempool] consensus pulling block size {:?}", transactions.len());
    let resp = GetBlockResponse { transactions };
    // send back to callback
    if let Err(e) = req.callback.send(Ok(resp)).map_err(|_| {
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
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
    mut consensus_requests: mpsc::Receiver<GetBlockRequest>,
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

    // Use a BoundedExecutor to restrict only `workers_available` concurrent
    // worker tasks that can process incoming transactions.
    let workers_available = smp.config.shared_mempool_max_concurrent_inbound_syncs;
    let bounded_executor = BoundedExecutor::new(workers_available, executor);

    loop {
        ::futures::select! {
            (mut msg, callback) = client_events.select_next_some() => {
                let now = Instant::now();
                bounded_executor
                .spawn(process_client_transaction_submission(
                    smp.clone(),
                    msg,
                    callback,
                ))
                .await;
                let duration = now.elapsed();
                if duration > Duration::from_millis(10) {
                    debug!("[shared mempool] client event took too long: {:?}", duration);
                }
            },
            msg = consensus_requests.select_next_some() => {
                debug!("[shared mempool] received consensus request");
                let now = Instant::now();
                process_consensus_request(smp.clone(), msg).await;
                let duration = now.elapsed();
                if duration > Duration::from_millis(10) {
                    debug!("[shared mempool] consensus event took too long: {:?}", duration);
                }
            }
            msg = state_sync_requests.select_next_some() => {
                debug!("[shared mempool] received state sync req");
                let now = Instant::now();
//                tokio::spawn(process_state_sync_request(smp.clone(), msg));
                process_state_sync_request(smp.clone(), msg).await;
                let duration = now.elapsed();
                if duration > Duration::from_millis(10) {
                    debug!("[shared mempool] state sync event took too long: {:?}", duration);
                }
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
                                let transactions: Vec<_> = msg
                                    .transactions
                                    .clone()
                                    .into_iter()
                                    .filter_map(|txn| convert_txn_from_proto(txn))
                                    .collect();
                                counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                                    .with_label_values(&["received".to_string().deref(), peer_id.to_string().deref()])
                                    .inc_by(transactions.len() as i64);
                                let smp_clone = smp.clone();
                                let timeline_state = match node_config.is_upstream_peer(peer_id, network_id) {
                                    true => TimelineState::NonQualified,
                                    false => TimelineState::NotReady,
                                };
                                let now = Instant::now();
                                bounded_executor
                                    .spawn(process_transaction_broadcast(
                                        smp_clone,
                                        transactions,
                                        timeline_state,
                                        peer_id
                                    ))
                                    .await;
                                let duration = now.elapsed();
                                if duration > Duration::from_millis(10) {
                                    debug!("[shared mempool] processing txn broadcast took too long: {:?}", duration);
                                }
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
    client_events: mpsc::Receiver<(
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
    consensus_requests: mpsc::Receiver<GetBlockRequest>,
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
    client_events: Receiver<(
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
    consensus_requests: Receiver<GetBlockRequest>,
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
