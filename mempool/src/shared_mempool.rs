// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    OP_COUNTERS,
};
use bounded_executor::BoundedExecutor;
use bytes05::Bytes;
use failure::format_err;
use futures::sync::mpsc::UnboundedSender;
use futures_preview::{
    channel::oneshot, compat::Future01CompatExt, future::join_all, Stream, StreamExt,
};
use libra_config::config::{MempoolConfig, NodeConfig};
use libra_logger::prelude::*;
use libra_prost_ext::MessageExt;
use libra_types::{transaction::SignedTransaction, PeerId};
use network::{
    proto::{
        BroadcastTransactionsRequest, BroadcastTransactionsResponse, MempoolSyncMsg,
        MempoolSyncMsg_oneof,
    },
    validator_network::{Event, MempoolNetworkEvents, MempoolNetworkSender, RpcError},
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
struct SharedMempool<V>
where
    V: TransactionValidation + 'static,
{
    mempool: Arc<Mutex<CoreMempool>>,
    network_sender: MempoolNetworkSender,
    config: MempoolConfig,
    storage_read_client: Arc<dyn StorageRead>,
    validator: Arc<V>,
    validator_peers: HashSet<PeerId>, // read-only and doesn't change, so concurrency-safe
    peer_info: Arc<Mutex<PeerInfo>>,
    subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
}

// TODO(gzh): Cannot derive `Clone`.
// Issue: https://github.com/rust-lang/rust/issues/26925
impl<V> Clone for SharedMempool<V>
where
    V: TransactionValidation + 'static,
{
    fn clone(&self) -> Self {
        Self {
            mempool: Arc::clone(&self.mempool),
            network_sender: self.network_sender.clone(),
            config: self.config.clone(),
            storage_read_client: Arc::clone(&self.storage_read_client),
            validator: Arc::clone(&self.validator),
            validator_peers: self.validator_peers.clone(),
            peer_info: self.peer_info.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
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
                let mut req = BroadcastTransactionsRequest::default();
                req.peer_id = peer_id.into();
                req.transactions = transactions
                    .into_iter()
                    .map(|txn| txn.try_into().unwrap())
                    .collect();

                let msg = MempoolSyncMsg {
                    message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(req)),
                };
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

/// used to validate incoming transactions and add them to local Mempool
async fn process_incoming_transactions<V>(
    smp: SharedMempool<V>,
    peer_id: PeerId,
    transactions: Vec<SignedTransaction>,
) where
    V: TransactionValidation,
{
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
                return Some((t, sequence_number, balance));
            }
            None
        })
        .collect();

    let validations = join_all(
        transactions
            .iter()
            .map(|t| smp.validator.validate_transaction(t.0.clone()).compat()),
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
                let insertion_result = mempool.add_txn(
                    transaction,
                    gas_cost,
                    sequence_number,
                    balance,
                    TimelineState::NonQualified,
                );
                OP_COUNTERS.inc(&format!(
                    "smp.transactions.status.{:?}.{:?}",
                    insertion_result.code, peer_id
                ));
            } else {
                OP_COUNTERS.inc(&format!(
                    "smp.transactions.status.validation_failed.{:?}",
                    peer_id
                ));
            }
        }
    }
    notify_subscribers(SharedMempoolNotification::NewTransactions, &smp.subscribers);
}

/// send callback here
async fn process_rpc_submit_transactions_request<V>(
    smp: SharedMempool<V>,
    peer_id: PeerId,
    request: BroadcastTransactionsRequest,
    callback: oneshot::Sender<Result<Bytes, RpcError>>,
) where
    V: TransactionValidation,
{
    // the RPC response object that will be returned to sender via callback
    let resp = match submit_transactions_to_mempool(smp, peer_id, request).await {
        Ok(response) => response,
        Err(e) => {
            error!(
                "[shared mempool] Error occurred in submitting transactions to local mempool: {:?}",
                e
            );
            // TODO return error in response
            let mut response = BroadcastTransactionsResponse::default();
            response.backpressure_ms = 0; // TODO this is placeholder
            response
        }
    };
    let response_msg = MempoolSyncMsg {
        message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsResponse(resp)),
    };

    // send response to callback
    let response_data = response_msg.to_bytes().expect("failed to serialize proto");
    if let Err(err) = callback
        .send(Ok(response_data))
        .map_err(|_| format_err!("[shared mempool] handling inbound RPC call timed out"))
    {
        error!(
            "[shared mempool] failed to process batched transaction request, error: {:?}",
            err
        );
    }
}

/// Tries to add transactions to local mempool
/// returns RPC response that can be sent to callback
/// peer_id is the PeerId of the node that sent the request
async fn submit_transactions_to_mempool<V>(
    smp: SharedMempool<V>,
    peer_id: PeerId,
    request: BroadcastTransactionsRequest,
) -> failure::Result<BroadcastTransactionsResponse>
where
    V: TransactionValidation,
{
    /////////////////////////////////////////////
    // convert from proto to SignedTransaction //
    /////////////////////////////////////////////
    let transactions: Vec<_> = request
        .transactions
        .clone()
        .into_iter()
        .filter_map(|txn| match SignedTransaction::try_from(txn.clone()) {
            Ok(t) => Some(t),
            Err(e) => {
                // TODO make RPC response for this invalid transaction
                // log error
                security_log(SecurityEvent::InvalidTransactionMP)
                    .error(&e)
                    .data(&txn)
                    .log();
                None
            }
        })
        .collect();

    //////////////////////////////////
    // validate transactions via VM //
    //////////////////////////////////
    let account_states = join_all(
        transactions
            .iter()
            .map(|t| get_account_state(smp.storage_read_client.clone(), t.sender())),
    )
    .await;

    let transactions: Vec<_> = transactions
        .into_iter()
        .enumerate()
        .filter_map(|(idx, t)| {
            if let Ok((sequence_number, balance)) = account_states[idx] {
                return Some((t, sequence_number, balance));
            }
            None
        })
        .collect();

    let validations = join_all(
        transactions
            .iter()
            .map(|t| smp.validator.validate_transaction(t.0.clone()).compat()),
    )
    .await;

    ///////////////////////////////
    // add txns to local mempool //
    ///////////////////////////////

    // The [`TimelineState`] for adding txns to mempool should only be TimelineState::NonQualified,
    // (i.e. this node should never try to broadcast it) if this MempoolSyncMsg came from one validator
    // to another validator in the same validator network.
    let timeline_state = if smp.validator_peers.contains(&peer_id) {
        TimelineState::NonQualified
    } else {
        TimelineState::NotReady
    };

    let mut mempool = smp
        .mempool
        .lock()
        .expect("[shared mempool] failed to acquire mempool lock");

    for (idx, (transaction, sequence_number, balance)) in transactions.into_iter().enumerate() {
        if let Ok(None) = validations[idx] {
            let gas_cost = transaction.max_gas_amount();
            mempool.add_txn(
                transaction,
                gas_cost,
                sequence_number,
                balance,
                // peer validator SMP network or from FN
                timeline_state,
            );
        // TODO log/update counters for MempoolAddTransactionStatus
        // TODO check for MempoolAddTransactionStatus::MempoolIsFull and calculate backpressure
        } else {
            // txn vm validation failed
            // TODO log/update counters for failed vm validation VMStatus
        }
    }

    //return RPC response for this request
    // TODO currently this is a dummy response - need to add real backpressure
    // and potentially more info on individual txn failures
    let mut response = BroadcastTransactionsResponse::default();
    response.backpressure_ms = 0;
    Ok(response)
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
    mut network_events: MempoolNetworkEvents,
) where
    V: TransactionValidation,
{
    let peer_info = smp.peer_info.clone();
    let subscribers = smp.subscribers.clone();

    // Use a BoundedExecutor to restrict only `workers_available` concurrent
    // worker tasks that can process incoming transactions.
    let workers_available = smp.config.shared_mempool_max_concurrent_inbound_syncs;
    let bounded_executor = BoundedExecutor::new(workers_available, executor);

    while let Some(event) = network_events.next().await {
        trace!("SharedMempoolEvent::NetworkEvent::{:?}", event);
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
                // TODO deprecate this in favor of Event::RpcRequest below
                Event::Message((peer_id, msg)) => {
                    OP_COUNTERS.inc("smp.event.message");
                    if let Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(request)) =
                        msg.message
                    {
                        let transactions: Vec<_> = request
                            .transactions
                            .clone()
                            .into_iter()
                            .filter_map(|txn| match SignedTransaction::try_from(txn.clone()) {
                                Ok(t) => Some(t),
                                Err(e) => {
                                    security_log(SecurityEvent::InvalidTransactionMP)
                                        .error(&e)
                                        .data(&txn)
                                        .log();
                                    None
                                }
                            })
                            .collect();
                        OP_COUNTERS.inc_by(
                            &format!("smp.transactions.received.{:?}", peer_id),
                            transactions.len(),
                        );
                        bounded_executor
                            .spawn(process_incoming_transactions(
                                smp.clone(),
                                peer_id,
                                transactions,
                            ))
                            .await;
                    }
                }
                Event::RpcRequest((peer_id, msg, callback)) => {
                    // handle rpc events for transactions
                    if let Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(request)) =
                        msg.message
                    {
                        // get transactions from MempoolSyncMsg
                        bounded_executor
                            .spawn(process_rpc_submit_transactions_request(
                                smp.clone(),
                                peer_id,
                                request,
                                callback,
                            ))
                            .await;
                    }
                }
            },
            Err(e) => {
                security_log(SecurityEvent::InvalidNetworkEventMP)
                    .error(&e)
                    .log();
            }
        }
    }
    crit!("SharedMempool inbound_network_task terminated");
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
    network_events: MempoolNetworkEvents,
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

    let validator_peers = if let Some(v) = &config.validator_network {
        v.network_peers.peers.iter().map(|(key, _)| *key).collect()
    } else {
        HashSet::new()
    };

    let smp = SharedMempool {
        mempool: mempool.clone(),
        config: config.mempool.clone(),
        network_sender,
        storage_read_client,
        validator,
        validator_peers,
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

    executor.spawn(inbound_network_task(smp, executor.clone(), network_events));

    executor.spawn(gc_task(
        mempool,
        config.mempool.system_transaction_gc_interval_ms,
    ));

    runtime
}
