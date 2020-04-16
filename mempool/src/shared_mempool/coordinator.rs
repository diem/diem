// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Processes that are directly spawned by shared mempool runtime initialization

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    counters,
    network::{MempoolNetworkEvents, MempoolSyncMsg},
    shared_mempool::{
        tasks,
        types::{notify_subscribers, IntervalStream, SharedMempool, SharedMempoolNotification},
    },
    CommitNotification, ConsensusRequest, SubmissionStatus,
};
use ::network::protocols::network::Event;
use anyhow::Result;
use bounded_executor::BoundedExecutor;
use channel::libra_channel;
use debug_interface::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    stream::select_all,
    StreamExt,
};
use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_security_logger::{security_log, SecurityEvent};
use libra_types::{on_chain_config::OnChainConfigPayload, transaction::SignedTransaction, PeerId};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{runtime::Handle, time::interval};
use vm_validator::vm_validator::TransactionValidation;

/// Coordinator that handles [`SyncEvent`], which is periodically emitted for us to
/// broadcast ready to go transactions to peers.
pub(crate) async fn broadcast_coordinator<V>(smp: SharedMempool<V>, mut interval: IntervalStream)
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
        tasks::sync_with_peers(&peer_info, &mempool, network_senders.clone(), batch_size).await;
        notify_subscribers(SharedMempoolNotification::Sync, &subscribers);
    }

    crit!("SharedMempool outbound_sync_task terminated");
}

/// Coordinator that handles inbound network events.
pub(crate) async fn request_coordinator<V>(
    smp: SharedMempool<V>,
    executor: Handle,
    network_events: Vec<(PeerId, MempoolNetworkEvents)>,
    mut client_events: mpsc::Receiver<(
        SignedTransaction,
        oneshot::Sender<Result<SubmissionStatus>>,
    )>,
    mut consensus_requests: mpsc::Receiver<ConsensusRequest>,
    mut state_sync_requests: mpsc::Receiver<CommitNotification>,
    mut mempool_reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
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
                trace_event!("mempool::client_event", {"txn", msg.sender(), msg.sequence_number()});
                bounded_executor
                .spawn(tasks::process_client_transaction_submission(
                    smp.clone(),
                    msg,
                    callback,
                ))
                .await;
            },
            msg = consensus_requests.select_next_some() => {
                tasks::process_consensus_request(smp.clone(), msg).await;
            }
            msg = state_sync_requests.select_next_some() => {
                tokio::spawn(tasks::process_state_sync_request(smp.clone(), msg));
            }
            config_update = mempool_reconfig_events.select_next_some() => {
                bounded_executor
                .spawn(tasks::process_config_update(config_update, smp.validator.clone()))
                .await;
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
                                    tasks::new_peer(&peer_info, peer_id, network_id);
                                }
                                notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                            }
                            Event::LostPeer(peer_id) => {
                                counters::SHARED_MEMPOOL_EVENTS
                                    .with_label_values(&["lost_peer".to_string().deref()])
                                    .inc();
                                if node_config.is_upstream_peer(peer_id, network_id) {
                                    tasks::lost_peer(&peer_info, peer_id);
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
                                            .spawn(tasks::process_transaction_broadcast(
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
                                        tasks::process_broadcast_ack(smp.clone(), start_id, end_id, is_validator);
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
pub(crate) async fn gc_coordinator(mempool: Arc<Mutex<CoreMempool>>, gc_interval_ms: u64) {
    let mut interval = interval(Duration::from_millis(gc_interval_ms));
    while let Some(_interval) = interval.next().await {
        mempool
            .lock()
            .expect("[shared mempool] failed to acquire mempool lock")
            .gc_by_system_ttl();
    }

    crit!("SharedMempool gc_task terminated");
}
