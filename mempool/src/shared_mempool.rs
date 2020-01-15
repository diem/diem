// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    OP_COUNTERS,
};
use admission_control_proto::{
    proto::admission_control::{
        submit_transaction_response::Status, SubmitTransactionRequest, SubmitTransactionResponse,
    },
    AdmissionControlStatus,
};
use anyhow::{format_err, Result};
use bounded_executor::BoundedExecutor;
use futures::{
    channel::{
        mpsc::{self, UnboundedSender},
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
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    ops::Deref,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};
use storage_client::StorageRead;
use tokio::{
    runtime::{Builder, Handle, Runtime},
    time::interval,
};
use vm_validator::vm_validator::{get_account_state, TransactionValidation};

/// state of last sync with peer
/// `timeline_id` is position in log of ready transactions
/// `is_alive` - is connection healthy
#[derive(Clone)]
struct PeerSyncState {
    timeline_id: u64,
    is_alive: bool,
}

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
    network_sender: MempoolNetworkSender,
    config: MempoolConfig,
    storage_read_client: Arc<dyn StorageRead>,
    validator: Arc<V>,
    validator_peers: HashSet<PeerId>, // read-only and immutable, so thread-safe
    peer_info: Arc<Mutex<PeerInfo>>,
    subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
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
fn new_peer(peer_info: &Mutex<PeerInfo>, peer_id: PeerId) {
    peer_info
        .lock()
        .expect("[shared mempool] failed to acquire peer_info lock")
        .entry(peer_id)
        .or_insert(PeerSyncState {
            timeline_id: 0,
            is_alive: true,
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
    network_sender: &'a mut MempoolNetworkSender,
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
                OP_COUNTERS.inc_by("smp.sync_with_peers", transactions.len());
                let mut msg = MempoolSyncMsg::default();
                msg.peer_id = peer_id.into();
                msg.transactions = transactions
                    .into_iter()
                    .map(|txn| txn.try_into().unwrap())
                    .collect();

                trace!(
                    "MempoolNetworkSender.send_to peer {} msg {:?}",
                    peer_id,
                    msg
                );
                // Since this is a direct-send, this will only error if the network
                // module has unexpectedly crashed or shutdown.
                network_sender
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

/// submits a list of SignedTransaction to the local mempool,
/// and returns an AdmissionControlStatus if `is_ac_endpoint_submission`
/// if `is_ac_endpoint_submission`, we expect a single transaction to be submitted
async fn process_incoming_transactions<V>(
    smp: SharedMempool<V>,
    peer_id: Option<PeerId>,
    transactions: Vec<SignedTransaction>,
) -> Option<Status>
where
    V: TransactionValidation,
{
    let (peer_id, is_ac_endpoint_submission, timeline_state) = match peer_id {
        Some(peer_id) => {
            if smp.validator_peers.contains(&peer_id) {
                (peer_id.to_string(), false, TimelineState::NonQualified)
            } else {
                (peer_id.to_string(), false, TimelineState::NotReady)
            }
        }
        None => ("client".to_string(), true, TimelineState::NotReady),
    };
    let mut status = None;

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
                status = Some(Status::VmStatus(VmStatusProto::from(
                    VMStatus::new(RESOURCE_DOES_NOT_EXIST)
                        .with_message("[shared mempool] failed to get account state".to_string()),
                )));
            }
            None
        })
        .collect();
    if is_ac_endpoint_submission && status.is_some() {
        return status;
    }

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
                OP_COUNTERS.inc(&format!(
                    "smp.transactions.status.{:?}.{:?}",
                    mempool_status.code, peer_id
                ));

                if is_ac_endpoint_submission {
                    if mempool_status.code == MempoolAddTransactionStatusCode::Valid {
                        status = Some(Status::AcStatus(AdmissionControlStatus::Accepted.into()));
                    } else {
                        status = Some(Status::MempoolStatus(
                            MempoolAddTransactionStatusProto::from(mempool_status),
                        ));
                    }
                }
            } else if let Ok(Some(validation_status)) = &validations[idx] {
                OP_COUNTERS.inc(&format!(
                    "smp.transactions.status.validation_failed.{:?}",
                    peer_id
                ));
                status = Some(Status::VmStatus(VmStatusProto::from(
                    validation_status.clone(),
                )));
            }
        }
    }

    if !is_ac_endpoint_submission {
        notify_subscribers(SharedMempoolNotification::NewTransactions, &smp.subscribers);
        None
    } else {
        status
    }
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

    let transaction = convert_txn_from_proto(txn_proto);
    // get status from attempt to submit txns
    match transaction {
        None => {
            response.status = Some(Status::AcStatus(
                AdmissionControlStatus::Rejected("submit txn rejected".to_string()).into(),
            ));
        }
        Some(txn) => {
            let result = process_incoming_transactions(smp.clone(), None, vec![txn]).await;
            match result {
                Some(status) => {
                    response.status = Some(status);
                }
                None => {
                    error!("[shared mempool] unexpected error happened");
                    // TODO set unexpected error status code in response
                }
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

/// This task handles [`SyncEvent`], which is periodically emitted for us to
/// broadcast ready to go transactions to peers.
async fn outbound_sync_task<V>(smp: SharedMempool<V>, mut interval: IntervalStream)
where
    V: TransactionValidation,
{
    let peer_info = smp.peer_info;
    let mempool = smp.mempool;
    let mut network_sender = smp.network_sender;
    let batch_size = smp.config.shared_mempool_batch_size;
    let subscribers = smp.subscribers;

    while let Some(sync_event) = interval.next().await {
        trace!("SyncEvent: {:?}", sync_event);
        sync_with_peers(&peer_info, &mempool, &mut network_sender, batch_size).await;
        notify_subscribers(SharedMempoolNotification::Sync, &subscribers);
    }

    crit!("SharedMempool outbound_sync_task terminated");
}

/// This task handles inbound network events.
async fn inbound_network_task<V>(
    smp: SharedMempool<V>,
    executor: Handle,
    network_events: Vec<MempoolNetworkEvents>,
    mut client_events: mpsc::Receiver<(
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
) where
    V: TransactionValidation,
{
    let peer_info = smp.peer_info.clone();
    let subscribers = smp.subscribers.clone();

    // Use a BoundedExecutor to restrict only `workers_available` concurrent
    // worker tasks that can process incoming transactions.
    let workers_available = smp.config.shared_mempool_max_concurrent_inbound_syncs;
    let bounded_executor = BoundedExecutor::new(workers_available, executor);

    let mut events = select_all(network_events).fuse();

    loop {
        ::futures::select! {
            // client events
            (mut msg, callback) = client_events.select_next_some() => {
                bounded_executor
                .spawn(process_client_transaction_submission(
                    smp.clone(),
                    msg,
                    callback,
                ))
                .await;
            }
            event = events.select_next_some() => {
                match event {
                    Ok(network_event) => match network_event {
                        Event::NewPeer(peer_id) => {
                            OP_COUNTERS.inc("smp.event.new_peer");
                            new_peer(&peer_info, peer_id);
                            notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                        }
                        Event::LostPeer(peer_id) => {
                            OP_COUNTERS.inc("smp.event.lost_peer");
                            lost_peer(&peer_info, peer_id);
                            notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                        }
                        Event::Message((peer_id, msg)) => {
                            OP_COUNTERS.inc("smp.event.message");
                            let transactions: Vec<_> = msg
                                .transactions
                                .clone()
                                .into_iter()
                                .filter_map(|txn| convert_txn_from_proto(txn))
                                .collect();
                            OP_COUNTERS.inc_by(
                                &format!("smp.transactions.received.{:?}", peer_id),
                                transactions.len(),
                            );
                            bounded_executor
                                .spawn(process_incoming_transactions(
                                    smp.clone(),
                                    Some(peer_id),
                                    transactions,
                                ))
                                .await;
                        }
                        _ => {
                            security_log(SecurityEvent::InvalidNetworkEventMP)
                                .error("UnexpectedNetworkEvent")
                                .data(&network_event)
                                .log();
                            debug_assert!(false, "Unexpected network event");
                        }
                    },
                    Err(e) => {
                        security_log(SecurityEvent::InvalidNetworkEventMP)
                            .error(&e)
                            .log();
                    }
                }
            }
        }
    }
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
    config: &NodeConfig,
    mempool: Arc<Mutex<CoreMempool>>,
    network_sender: MempoolNetworkSender,
    network_events: Vec<MempoolNetworkEvents>,
    ac_client_events: mpsc::Receiver<(
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
    storage_read_client: Arc<dyn StorageRead>,
    validator: Arc<V>,
    subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
    timer: Option<IntervalStream>,
) -> Runtime
where
    V: TransactionValidation + 'static,
{
    let runtime = Builder::new()
        .thread_name("shared-mem-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[shared mempool] failed to create runtime");
    let executor = runtime.handle();

    let peer_info = Arc::new(Mutex::new(PeerInfo::new()));
    let validator_peers;
    match &config.validator_network {
        Some(v) => {
            validator_peers = v.network_peers.peers.iter().map(|(key, _)| *key).collect();
        }
        None => {
            validator_peers = HashSet::new();
        }
    }

    let smp = SharedMempool {
        mempool: mempool.clone(),
        config: config.mempool.clone(),
        network_sender,
        storage_read_client,
        validator,
        peer_info,
        validator_peers,
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
        network_events,
        ac_client_events,
    ));

    executor.spawn(gc_task(
        mempool,
        config.mempool.system_transaction_gc_interval_ms,
    ));

    runtime
}
