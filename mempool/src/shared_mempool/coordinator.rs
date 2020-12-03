// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Processes that are directly spawned by shared mempool runtime initialization

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    counters,
    logging::{LogEntry, LogEvent, LogSchema},
    network::{MempoolNetworkEvents, MempoolSyncMsg},
    shared_mempool::{
        tasks,
        types::{notify_subscribers, SharedMempool, SharedMempoolNotification},
    },
    CommitNotification, ConsensusRequest, SubmissionStatus,
};
use ::network::protocols::network::Event;
use anyhow::Result;
use bounded_executor::BoundedExecutor;
use channel::diem_channel;
use diem_config::{config::PeerNetworkId, network_id::NodeNetworkId};
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use diem_trace::prelude::*;
use diem_types::{on_chain_config::OnChainConfigPayload, transaction::SignedTransaction};
use futures::{
    channel::{mpsc, oneshot},
    stream::{select_all, FuturesUnordered},
    StreamExt,
};
use std::{
    ops::Deref,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{runtime::Handle, time::interval};
use vm_validator::vm_validator::TransactionValidation;

/// Coordinator that handles inbound network events and outbound txn broadcasts.
pub(crate) async fn coordinator<V>(
    mut smp: SharedMempool<V>,
    executor: Handle,
    network_events: Vec<(NodeNetworkId, MempoolNetworkEvents)>,
    mut client_events: mpsc::Receiver<(
        SignedTransaction,
        oneshot::Sender<Result<SubmissionStatus>>,
    )>,
    mut consensus_requests: mpsc::Receiver<ConsensusRequest>,
    mut state_sync_requests: mpsc::Receiver<CommitNotification>,
    mut mempool_reconfig_events: diem_channel::Receiver<(), OnChainConfigPayload>,
) where
    V: TransactionValidation,
{
    info!(LogSchema::event_log(
        LogEntry::CoordinatorRuntime,
        LogEvent::Start
    ));
    let smp_events: Vec<_> = network_events
        .into_iter()
        .map(|(network_id, events)| events.map(move |e| (network_id.clone(), e)))
        .collect();
    let mut events = select_all(smp_events).fuse();
    let mempool = smp.mempool.clone();
    let peer_manager = smp.peer_manager.clone();
    let subscribers = &mut smp.subscribers.clone();
    let mut scheduled_broadcasts = FuturesUnordered::new();

    // Use a BoundedExecutor to restrict only `workers_available` concurrent
    // worker tasks that can process incoming transactions.
    let workers_available = smp.config.shared_mempool_max_concurrent_inbound_syncs;
    let bounded_executor = BoundedExecutor::new(workers_available, executor.clone());

    loop {
        let _timer = counters::MAIN_LOOP.start_timer();
        ::futures::select! {
            (msg, callback) = client_events.select_next_some() => {
                trace_event!("mempool::client_event", {"txn", msg.sender(), msg.sequence_number()});
                // this timer measures how long it took for the bounded executor to *schedule* the
                // task
                let _timer = counters::TASK_SPAWN_LATENCY
                    .with_label_values(&[counters::CLIENT_EVENT_LABEL, counters::SPAWN_LABEL])
                    .start_timer();
                // this timer measures how long it took for the task to go from scheduled to started
                let task_start_timer = counters::TASK_SPAWN_LATENCY
                    .with_label_values(&[counters::CLIENT_EVENT_LABEL, counters::START_LABEL])
                    .start_timer();
                bounded_executor
                .spawn(tasks::process_client_transaction_submission(
                    smp.clone(),
                    msg,
                    callback,
                    task_start_timer,
                ))
                .await;
            },
            msg = consensus_requests.select_next_some() => {
                tasks::process_consensus_request(&mempool, msg).await;
            }
            msg = state_sync_requests.select_next_some() => {
                let _timer = counters::TASK_SPAWN_LATENCY
                    .with_label_values(&[counters::STATE_SYNC_EVENT_LABEL, counters::SPAWN_LABEL])
                    .start_timer();
                tokio::spawn(tasks::process_state_sync_request(mempool.clone(), msg));
            }
            config_update = mempool_reconfig_events.select_next_some() => {
                info!(LogSchema::event_log(LogEntry::ReconfigUpdate, LogEvent::Received));
                let _timer = counters::TASK_SPAWN_LATENCY
                    .with_label_values(&[counters::RECONFIG_EVENT_LABEL, counters::SPAWN_LABEL])
                    .start_timer();
                bounded_executor
                    .spawn(tasks::process_config_update(config_update, smp.validator.clone()))
                    .await;
            },
            (peer, backoff) = scheduled_broadcasts.select_next_some() => {
                tasks::execute_broadcast(peer, backoff, &mut smp, &mut scheduled_broadcasts, executor.clone());
            },
            (network_id, event) = events.select_next_some() => {
                match event {
                    Event::NewPeer(peer_id, origin) => {
                        counters::SHARED_MEMPOOL_EVENTS
                            .with_label_values(&["new_peer".to_string().deref()])
                            .inc();
                        let peer = PeerNetworkId(network_id, peer_id);
                        let is_new_peer = peer_manager.add_peer(peer.clone(), origin);
                        let is_upstream_peer = peer_manager.is_upstream_peer(&peer, Some(origin));
                        debug!(LogSchema::new(LogEntry::NewPeer).peer(&peer).is_upstream_peer(is_upstream_peer));
                        notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                        if is_new_peer && is_upstream_peer {
                            tasks::execute_broadcast(peer, false, &mut smp, &mut scheduled_broadcasts, executor.clone());
                        }
                    }
                    Event::LostPeer(peer_id, origin) => {
                        counters::SHARED_MEMPOOL_EVENTS
                            .with_label_values(&["lost_peer".to_string().deref()])
                            .inc();
                        let peer = PeerNetworkId(network_id, peer_id);
                        debug!(LogSchema::new(LogEntry::LostPeer)
                            .peer(&peer)
                            .is_upstream_peer(peer_manager.is_upstream_peer(&peer, Some(origin)))
                        );
                        peer_manager.disable_peer(peer);
                        notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                    }
                    Event::Message(peer_id, msg) => {
                        counters::SHARED_MEMPOOL_EVENTS
                            .with_label_values(&["message".to_string().deref()])
                            .inc();
                        match msg {
                            MempoolSyncMsg::BroadcastTransactionsRequest{request_id, transactions} => {
                                let smp_clone = smp.clone();
                                let peer = PeerNetworkId(network_id, peer_id);
                                let timeline_state = match peer_manager
                                    .is_upstream_peer(&peer, None)
                                {
                                    true => TimelineState::NonQualified,
                                    false => TimelineState::NotReady,
                                };
                                // this timer measures how long it took for the bounded executor to
                                // *schedule* the task
                                let _timer = counters::TASK_SPAWN_LATENCY
                                    .with_label_values(&[counters::PEER_BROADCAST_EVENT_LABEL, counters::SPAWN_LABEL])
                                    .start_timer();
                                // this timer measures how long it took for the task to go from scheduled
                                // to started
                                let task_start_timer = counters::TASK_SPAWN_LATENCY
                                    .with_label_values(&[counters::PEER_BROADCAST_EVENT_LABEL, counters::START_LABEL])
                                    .start_timer();
                                bounded_executor
                                    .spawn(tasks::process_transaction_broadcast(
                                        smp_clone,
                                        transactions,
                                        request_id,
                                        timeline_state,
                                        peer,
                                        task_start_timer
                                    ))
                                    .await;
                            }
                            MempoolSyncMsg::BroadcastTransactionsResponse{request_id, retry, backoff} => {
                                let ack_timestamp = SystemTime::now();
                                peer_manager.process_broadcast_ack(PeerNetworkId(network_id.clone(), peer_id), request_id, retry, backoff, ack_timestamp);
                            }
                        }
                    }
                    Event::RpcRequest(peer_id, _msg, _res_tx) => {
                        counters::UNEXPECTED_NETWORK_MSG_COUNT
                            .with_label_values(&[&network_id.network_id().to_string(), &peer_id.to_string()])
                            .inc();
                        sample!(
                            SampleRate::Duration(Duration::from_secs(60)),
                            warn!(
                                LogSchema::new(LogEntry::UnexpectedNetworkMsg)
                                    .peer(&PeerNetworkId(network_id, peer_id))
                            )
                        );
                    }
                }
            },
            complete => break,
        }
    }
    error!(LogSchema::event_log(
        LogEntry::CoordinatorRuntime,
        LogEvent::Terminated
    ));
}

/// GC all expired transactions by SystemTTL
pub(crate) async fn gc_coordinator(mempool: Arc<Mutex<CoreMempool>>, gc_interval_ms: u64) {
    info!(LogSchema::event_log(LogEntry::GCRuntime, LogEvent::Start));
    let mut interval = interval(Duration::from_millis(gc_interval_ms));
    while let Some(_interval) = interval.next().await {
        sample!(
            SampleRate::Duration(Duration::from_secs(60)),
            info!(LogSchema::event_log(LogEntry::GCRuntime, LogEvent::Live))
        );
        mempool.lock().gc();
    }

    error!(LogSchema::event_log(
        LogEntry::GCRuntime,
        LogEvent::Terminated
    ));
}

/// Periodically logs a snapshot of transactions in core mempool
/// In the future we may want a interactive way to directly query mempool's internal state
/// For now, we will rely on this periodic snapshot to observe the internal state
pub(crate) async fn snapshot_job(mempool: Arc<Mutex<CoreMempool>>, snapshot_interval_secs: u64) {
    let mut interval = interval(Duration::from_secs(snapshot_interval_secs));
    while let Some(_interval) = interval.next().await {
        let snapshot = mempool.lock().gen_snapshot();
        debug!(LogSchema::new(LogEntry::MempoolSnapshot).txns(snapshot));
    }
}
