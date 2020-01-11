// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    OP_COUNTERS,
};
use bounded_executor::BoundedExecutor;
use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    future::join_all,
    //    Stream,
    StreamExt,
};
use libra_config::config::{MempoolConfig, NodeConfig};
use libra_logger::prelude::*;
use libra_types::{transaction::SignedTransaction, PeerId};
use network::{
    proto::{
        BroadcastTransactionsRequest, BroadcastTransactionsResponse,
        BroadcastTransactionsStatusCode, MempoolSyncMsg, MempoolSyncMsg_oneof,
    },
    validator_network::{Event, MempoolNetworkEvents, MempoolNetworkSender},
};
use std::{
    collections::{BTreeMap, HashMap},
    convert::{TryFrom, TryInto},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use storage_client::StorageRead;
use tokio::{
    runtime::{Builder, Handle, Runtime},
    time::interval,
};
use vm_validator::vm_validator::{get_account_state, TransactionValidation};

enum ChannelStatus {
    CLOSED,
    OPEN,
}

#[derive(Clone, Copy)]
enum ScheduledEvent {
    Broadcast,
    CheckBroadcast,
}

#[derive(Clone, Copy)]
struct ScheduledEventInfo {
    peer_id: PeerId,
    event_type: ScheduledEvent,
}

/// state of last broadcast with peer
/// `timeline_id` is position in log of ready transactions
/// `is_alive` - is connection healthy
struct BroadcastUpdate {
    peer_id: PeerId,
    is_alive: bool,
}

/// state of last broadcast with peer
/// `num_batches_sent` - number of broadcasts sent to a peer that are pending an ACK response
/// `timeline_id` - position in log of ready transactions
/// `is_alive` - is connection healthy
#[derive(Debug)]
struct PeerBroadcastState {
    num_batches_sent: usize,
    txn_timeline_id: u64,
    is_alive: bool,
    broadcast_times: Vec<Duration>, // timestamp of broadcasts, in order
}

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
                if t.sequence_number() >= sequence_number {
                    return Some((t, sequence_number, balance));
                }
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
    // send back to direct_send

    let mut resp = BroadcastTransactionsResponse::default();
    // TODO send back accurate code back
    resp.set_code(BroadcastTransactionsStatusCode::Success);
    let msg = MempoolSyncMsg {
        message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsResponse(resp)),
    };
    let mut network_sender = smp.network_sender;
    debug!("[shared mempool] direct send response to {:?}", peer_id);
    network_sender
        .send_to(peer_id, msg)
        .await
        .expect("[shared mempool] failed to direct-send mempool broadcast message");
}

async fn broadcast_transactions<V>(
    peer_id: PeerId,
    transactions: Vec<SignedTransaction>,
    smp: SharedMempool<V>,
) where
    V: TransactionValidation,
{
    if transactions.is_empty() {
        return;
    }

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

    let mut network_sender = smp.network_sender;
    let subscribers = smp.subscribers;
    network_sender
        .send_to(peer_id, msg)
        .await
        .expect("[shared mempool] failed to direct-send mempool broadcast message");
    notify_subscribers(SharedMempoolNotification::Sync, &subscribers);
}

/// returns earliest duration in the ordered keyset of `b_tree`
fn pop_btree(
    b_tree: &BTreeMap<Duration, ScheduledEventInfo>,
) -> Option<(Duration, ScheduledEventInfo)> {
    match b_tree.iter().next() {
        Some((timestamp, scheduled_event)) => Some((*timestamp, *scheduled_event)),
        None => None,
    }
}

async fn broadcast_task<V>(
    smp: SharedMempool<V>,
    mut broadcast_trigger: UnboundedReceiver<BroadcastUpdate>,
) where
    V: TransactionValidation,
{
    debug!("[shared mempool] broadcast_task started");
    // a BTreeMap whose (key, val) = (earliest scheduled attempted broadcast timestamp for peer, peer_id)
    // there should only be one entry per peer_id value
    let mut scheduled_events: BTreeMap<Duration, ScheduledEventInfo> = BTreeMap::new();

    // a HashMap where (key, val) = (
    //     peer_id,
    //     (num of batches sent so far,
    //     time of earliest scheduled attempted broadcast time in b_tree,
    //     timeline id of transaction last sent
    //     )
    // )
    let mut peer_batch_broadcast_state: HashMap<PeerId, PeerBroadcastState> = HashMap::new();

    let batch_size = smp.config.shared_mempool_batch_size;
    let max_batch_count: usize = 15; // TODO change this

    loop {
        // loop for processing scheduled broadcasts
        let mut next_scheduled_broadcast_timestamp = None;

        // process scheduled broadcasts
        loop {
            // debug!("[shared mempool] processing scheduled broadcasts");
            match pop_btree(&scheduled_events) {
                None => {
                    debug!("[shared mempool] b tree empty, no scheduled broadcasts");
                    break;
                }
                Some((timestamp, scheduled_event)) => {
                    // prune BTreeMap of processed scheduled broadcasts
                    debug!("[shared mempool] processing scheduled broadcasts for timestamp");
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap();
                    if timestamp > now {
                        debug!("[shared mempool] too early, break");
                        next_scheduled_broadcast_timestamp = Some(timestamp);
                        break;
                    }

                    scheduled_events.remove(&timestamp);

                    let peer_id = scheduled_event.peer_id;
                    // attempt this scheduled broadcast if it is ready
                    let batch_broadcast_state =
                        peer_batch_broadcast_state.get_mut(&peer_id).unwrap();
                    debug!("[shared mempool] state: {:?}", batch_broadcast_state);
                    match scheduled_event.event_type {
                        ScheduledEvent::Broadcast => {
                            // attempt this scheduled broadcast if it is ready
                            if batch_broadcast_state.is_alive {
                                let timeline_id = batch_broadcast_state.txn_timeline_id;
                                debug!(
                                    "[shared mempool] reading txn for peer {:?} and timeline id {:?}",
                                    peer_id, timeline_id
                                );
                                let (transactions, new_timeline_id) = smp
                                    .mempool
                                    .lock()
                                    .expect("[shared mempool] failed to acquire mempool lock")
                                    .read_timeline(timeline_id, batch_size);

                                debug!(
                                    "[shared mempool] got transaction for peer {:?} for timeline_id {:?}",
                                    peer_id, timeline_id
                                );

                                let rescheduled_timestamp = timestamp
                                    .checked_add(Duration::new(0, 50 * 1_000_000))
                                    .unwrap();
                                let scheduled_broadcast = ScheduledEventInfo {
                                    peer_id,
                                    event_type: ScheduledEvent::Broadcast,
                                };
                                if transactions.is_empty() {
                                    // reschedule this broadcast to 50 ms later, when there might be actual txns to broadcast=
                                    debug!("[shared mempool] no txns to broadcast, rescheduling");
                                    scheduled_events
                                        .insert(rescheduled_timestamp, scheduled_broadcast);
                                } else {
                                    // broadcast
                                    let smp_clone = smp.clone();
                                    debug!("[shared mempool] broadcasting txns to {:?}", peer_id);
                                    broadcast_transactions(peer_id, transactions, smp_clone).await;

                                    // update states
                                    batch_broadcast_state.num_batches_sent += 1;
                                    batch_broadcast_state.txn_timeline_id = new_timeline_id;
                                    batch_broadcast_state.broadcast_times.push(timestamp);

                                    // schedule the expiration to hear back for this broadcast
                                    let scheduled_staleness_check = ScheduledEventInfo {
                                        peer_id,
                                        event_type: ScheduledEvent::CheckBroadcast,
                                    };
                                    let staleness_check_timestamp =
                                        timestamp.checked_add(Duration::new(60, 0)).unwrap();
                                    scheduled_events.insert(
                                        staleness_check_timestamp,
                                        scheduled_staleness_check,
                                    );

                                    // reschedule the x + 1'th batch if x < max_batch_count
                                    if batch_broadcast_state.num_batches_sent < max_batch_count {
                                        scheduled_events
                                            .insert(rescheduled_timestamp, scheduled_broadcast);
                                    }
                                }
                            } else {
                                debug!(
                                    "[shared mempool] skipping scheduled broadcasts of lost peer {:?}",
                                    peer_id,
                                );
                            }
                        }
                        ScheduledEvent::CheckBroadcast => {
                            // timestamp = the time a batch expires
                            // this check itself might have been expired
                            debug!(
                                "[shared mempool] checking broadcast expiration for {:?}: ",
                                peer_id
                            );

                            let first_broadcast_expiration_timestamp = batch_broadcast_state
                                .broadcast_times[0]
                                .checked_add(Duration::from_secs(60))
                                .unwrap();
                            if timestamp == first_broadcast_expiration_timestamp {
                                debug!("[shared mempool] broadcast expired for {:?}", peer_id);
                                batch_broadcast_state.num_batches_sent -= 1;
                                batch_broadcast_state.broadcast_times.remove(0);

                                // need to update txn_timeline_id

                                let resend_time = batch_broadcast_state
                                    .broadcast_times
                                    .last()
                                    .unwrap()
                                    .checked_add(Duration::from_millis(50))
                                    .unwrap();

                                // reschedule broadcast if stale
                                scheduled_events.insert(
                                    resend_time,
                                    ScheduledEventInfo {
                                        peer_id,
                                        event_type: ScheduledEvent::Broadcast,
                                    },
                                );
                            }
                        }
                    }
                    //                    // attempt this scheduled broadcast if it is ready
                    //                    let batch_broadcast_state =
                    //                        peer_batch_broadcast_state.get_mut(&peer_id).unwrap();
                    //                    if batch_broadcast_state.is_alive {
                    //                        let timeline_id = batch_broadcast_state.txn_timeline_id;
                    //                        println!(
                    //                            "[shared mempool] reading txn for peer {:?} and timeline id {:?}",
                    //                            peer_id, timeline_id
                    //                        );
                    //                        let (transactions, new_timeline_id) = smp
                    //                            .mempool
                    //                            .lock()
                    //                            .expect("[shared mempool] failed to acquire mempool lock")
                    //                            .read_timeline(timeline_id, batch_size);
                    //
                    //                        println!(
                    //                            "[shared mempool] got transaction for peer {:?} for timeline_id {:?}",
                    //                            peer_id, timeline_id
                    //                        );
                    //
                    //                        let rescheduled_timestamp = timestamp
                    //                            .checked_add(Duration::new(0, 50 * 1_000_000))
                    //                            .unwrap();
                    //                        let scheduled_broadcast = ScheduledEventInfo {
                    //                            peer_id,
                    //                            event_type: ScheduledEvent::BROADCAST,
                    //                        };
                    //                        if transactions.is_empty() {
                    //                            // reschedule this broadcast to 50 ms later, when there might be actual txns to broadcast=
                    //                            scheduled_events.insert(rescheduled_timestamp, scheduled_broadcast);
                    //                        } else {
                    //                            // broadcast
                    //                            let smp_clone = smp.clone();
                    //                            println!("[shared mempool] broadcasting txns");
                    //                            broadcast_transactions(peer_id, transactions, smp_clone).await;
                    //
                    //                            // update states
                    //                            batch_broadcast_state.num_batches_sent += 1;
                    //                            batch_broadcast_state.txn_timeline_id = new_timeline_id;
                    //                            batch_broadcast_state.broadcast_times.push(timestamp);
                    //
                    //                            // schedule the expiration to hear back for this broadcast
                    //                            let scheduled_staleness_check = ScheduledEventInfo {
                    //                                peer_id,
                    //                                event_type: ScheduledEvent::CHECK_BROADCAST,
                    //                            };
                    //                            let staleness_check_timestamp = timestamp
                    //                                .checked_add(Duration::new(60, 0))
                    //                                .unwrap();
                    //                            scheduled_events.insert(staleness_check_timestamp, scheduled_staleness_check);
                    //
                    //                            // reschedule the x + 1'th batch if x < max_batch_count
                    //                            if batch_broadcast_state.num_batches_sent < max_batch_count {
                    //                                scheduled_events.insert(rescheduled_timestamp, scheduled_broadcast);
                    //                            }
                    //                        }
                    //                    } else {
                    //                        println!("[shared mempool] skipping scheduled broadcasts of lost peers");
                    //                    }
                }
            }
        }

        // listen to broadcast_triggers
        let listen_broadcast_triggers: ChannelStatus = async {
            debug!("[shared mempool] in listen_broadcast_triggers");

            loop {
                // debug!("[shared mempool] in loop");
                ::futures::select! {
                    update = broadcast_trigger.next() => {
                        match update {
                            Some(update) => {
                                // debug!("[shared mempool] received broadcast trigger");
                                let peer_id = update.peer_id;
                                let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

                                if !update.is_alive {
                                    match peer_batch_broadcast_state.get_mut(&peer_id) {
                                        Some(entry) => {
                                            entry.is_alive = false;
                                            debug!("[shared mempool] marked peer as lost {:?}", entry);
                                        }
                                        None => {
                                            debug!("[shared mempool] received LostPeer notification for non-existent peer");
                                        }
                                    }
                                } else {
                                    // NewPeer being revived
                                    // BroadcastResponse for already alive peer

                                    // update state that ack for this broadcast was received
                                    // schedule next broadcast

                                    scheduled_events.insert(now, ScheduledEventInfo {
                                        peer_id,
                                        event_type: ScheduledEvent::Broadcast,
                                    });
                                    debug!("[shared mempool] scheduled broadcast for {:?} for {:?}", now, peer_id);

                                    match peer_batch_broadcast_state.get_mut(&peer_id) {
                                        Some(entry) => {
                                            if entry.is_alive {
                                                entry.num_batches_sent -= 1;
                                                entry.broadcast_times.remove(0);
                                            }
                                            entry.is_alive = true;
                                        }
                                        None => {
                                            // brand new peer
                                            peer_batch_broadcast_state.insert(peer_id, PeerBroadcastState {
                                                num_batches_sent: 0,
                                                is_alive: true,
                                                txn_timeline_id: 0,
                                                broadcast_times: vec![],
                                            });
                                        }
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    default => {
                        // debug!("[shared mempool] in default");
                        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
                        if let Some(deadline) = next_scheduled_broadcast_timestamp {
                            if now >= deadline {
                                debug!("[shared mempool] time's up in listen_broadcast_triggers");
                                break;
                            }
                        } else {
                            // since we received a peer, we can start looping through btree
                            debug!("[shared mempool] we got a peer, so break");
                            break;
                        }
                    }
                    complete => {
                        debug!("[shared mempool] completed yeh");
                        return ChannelStatus::CLOSED;
                    }
                }
            }

            debug!("[shared mempool] finished listen_broadcast_triggers");
            ChannelStatus::OPEN
        }.await;

        // need this for graceful exit if senders are all dropped
        if let ChannelStatus::CLOSED = listen_broadcast_triggers {
            trace!("[shared mempool] stream has been exhausted, closing");
            return;
        }
    }
}

/// This task handles inbound network events.
async fn inbound_network_task<V>(
    smp: SharedMempool<V>,
    executor: Handle,
    mut network_events: MempoolNetworkEvents,
    broadcast_trigger: UnboundedSender<BroadcastUpdate>,
) where
    V: TransactionValidation,
{
    debug!("[shared mempool] in inbound_network_task");
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
                    debug!("[shared mempool] received NEW PEER {:?}", peer_id);
                    OP_COUNTERS.inc("smp.event.new_peer");
                    // also consider the case where the this is not a completely new peer (new --> lost --> new)
                    let update = BroadcastUpdate {
                        peer_id,
                        is_alive: true,
                    };
                    broadcast_trigger.clone().unbounded_send(update).unwrap();
                    debug!("[shared mempool] sent update to broadcast_trigger");
                    notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                }
                Event::LostPeer(peer_id) => {
                    OP_COUNTERS.inc("smp.event.lost_peer");
                    notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                    let update = BroadcastUpdate {
                        peer_id,
                        is_alive: false,
                    };
                    broadcast_trigger.clone().unbounded_send(update).unwrap();
                    debug!("[shared mempool] sent update to broadcast_trigger");
                    notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                }
                Event::Message((peer_id, msg)) => {
                    match msg.clone().message {
                        Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(req)) => {
                            // process incoming
                            let transactions = req
                                .transactions
                                .clone()
                                .into_iter()
                                .filter_map(|txn| match SignedTransaction::try_from(txn) {
                                    Ok(t) => Some(t),
                                    Err(e) => {
                                        security_log(SecurityEvent::InvalidTransactionMP)
                                            .error(&e)
                                            .data(&msg)
                                            .log();
                                        None
                                    }
                                })
                                .collect();

                            debug!("[shared mempool] spawning process_incoming_transactions");
                            bounded_executor
                                .spawn(process_incoming_transactions(
                                    smp.clone(),
                                    peer_id,
                                    transactions,
                                ))
                                .await;
                            debug!("[shared mempool] spawned process_incoming_transactions");
                        }
                        Some(MempoolSyncMsg_oneof::BroadcastTransactionsResponse(_response)) => {
                            // send to channel to
                            debug!("[shared mempool] received BroadcastTransactionsResponse from peer {:?}", peer_id);
                            let update = BroadcastUpdate {
                                peer_id,
                                is_alive: true,
                            };
                            broadcast_trigger.clone().unbounded_send(update).expect("[shared mempool] broadcast update channel send to outbound task failed");
                        }
                        _ => {
                            trace!("[shared mempool] received empty payload");
                        }
                    }
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
    //    timer: Option<IntervalStream>,
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

    let smp = SharedMempool {
        mempool: mempool.clone(),
        config: config.mempool.clone(),
        network_sender,
        storage_read_client,
        validator,
        subscribers,
    };

    let smp_broadcast = smp.clone();

    let (sender, receiver) = unbounded();

    executor.spawn(broadcast_task(smp_broadcast, receiver));

    executor.spawn(inbound_network_task(
        smp,
        executor.clone(),
        network_events,
        sender,
    ));

    executor.spawn(gc_task(
        mempool,
        config.mempool.system_transaction_gc_interval_ms,
    ));

    runtime
}
