// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Processes that are directly spawned by shared mempool runtime initialization

use crate::{
    core_mempool::{CoreMempool, TimelineState},
    counters,
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
use channel::libra_channel;
use futures::{
    channel::{mpsc, oneshot},
    stream::{select_all, FuturesUnordered},
    StreamExt,
};
use libra_config::{config::PeerNetworkId, network_id::NodeNetworkId};
use libra_logger::prelude::*;
use libra_trace::prelude::*;
use libra_types::{on_chain_config::OnChainConfigPayload, transaction::SignedTransaction};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
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
    mut mempool_reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
) where
    V: TransactionValidation,
{
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
        ::futures::select! {
            (mut msg, callback) = client_events.select_next_some() => {
                trace_event!("mempool::client_event", {"txn", msg.sender(), msg.sequence_number()});
                let _ = counters::TASK_SPAWN_LATENCY
                .with_label_values(&[counters::CLIENT_EVENT_LABEL])
                .start_timer();
                bounded_executor
                .spawn(tasks::process_client_transaction_submission(
                    smp.clone(),
                    msg,
                    callback,
                ))
                .await;
            },
            msg = consensus_requests.select_next_some() => {
                tasks::process_consensus_request(&mempool, msg).await;
            }
            msg = state_sync_requests.select_next_some() => {
                let _ = counters::TASK_SPAWN_LATENCY
                    .with_label_values(&[counters::STATE_SYNC_EVENT_LABEL])
                    .start_timer();
                tokio::spawn(tasks::process_state_sync_request(mempool.clone(), msg));
            }
            config_update = mempool_reconfig_events.select_next_some() => {
                let _ = counters::TASK_SPAWN_LATENCY
                    .with_label_values(&[counters::RECONFIG_EVENT_LABEL])
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
                    Ok(network_event) => {
                        match network_event {
                            Event::NewPeer(peer_id, origin) => {
                                counters::SHARED_MEMPOOL_EVENTS
                                    .with_label_values(&["new_peer".to_string().deref()])
                                    .inc();
                                let peer = PeerNetworkId(network_id, peer_id);
                                let is_new_peer = peer_manager.add_peer(peer.clone(), origin);
                                notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                                if is_new_peer && peer_manager.is_upstream_peer(&peer, None) {
                                    tasks::execute_broadcast(peer, false, &mut smp, &mut scheduled_broadcasts, executor.clone());
                                }
                            }
                            Event::LostPeer(peer_id, _origin) => {
                                counters::SHARED_MEMPOOL_EVENTS
                                    .with_label_values(&["lost_peer".to_string().deref()])
                                    .inc();
                                peer_manager.disable_peer(PeerNetworkId(network_id, peer_id));
                                notify_subscribers(SharedMempoolNotification::PeerStateChange, &subscribers);
                            }
                            Event::Message((peer_id, msg)) => {
                                counters::SHARED_MEMPOOL_EVENTS
                                    .with_label_values(&["message".to_string().deref()])
                                    .inc();
                                match msg {
                                    MempoolSyncMsg::BroadcastTransactionsRequest{request_id, transactions} => {
                                        counters::SHARED_MEMPOOL_TRANSACTIONS_PROCESSED
                                            .with_label_values(&["received".to_string().deref(), peer_id.to_string().deref()])
                                            .inc_by(transactions.len() as i64);
                                        let smp_clone = smp.clone();
                                        let peer = PeerNetworkId(network_id, peer_id);
                                        let timeline_state = match peer_manager
                                            .is_upstream_peer(&peer, None)
                                        {
                                            true => TimelineState::NonQualified,
                                            false => TimelineState::NotReady,
                                        };
                                        let _ = counters::TASK_SPAWN_LATENCY
                                            .with_label_values(&[counters::PEER_BROADCAST_EVENT_LABEL])
                                            .start_timer();
                                        bounded_executor
                                            .spawn(tasks::process_transaction_broadcast(
                                                smp_clone,
                                                transactions,
                                                request_id,
                                                timeline_state,
                                                peer
                                            ))
                                            .await;
                                    }
                                    MempoolSyncMsg::BroadcastTransactionsResponse{request_id, retry_txns, backoff} => {
                                        let ack_timestamp = Instant::now();
                                        peer_manager.process_broadcast_ack(PeerNetworkId(network_id.clone(), peer_id), request_id, retry_txns, backoff, ack_timestamp);
                                    }
                                };
                            }
                            Event::RpcRequest((peer_id, msg, res_tx)) => {
                                error!(
                                    SecurityEvent::InvalidNetworkEventMempool,
                                    message = msg,
                                    peer_id = peer_id,
                                );
                                debug_assert!(false, "Unexpected network event rpc request");
                            }
                        }
                    },
                    Err(e) => {
                        error!(
                            SecurityEvent::InvalidNetworkEventMempool,
                            error = e.to_string()
                        );
                    }
                };
            },
            complete => break,
        }
    }
    error!("[shared mempool] inbound_network_task terminated");
}

/// GC all expired transactions by SystemTTL
pub(crate) async fn gc_coordinator(mempool: Arc<Mutex<CoreMempool>>, gc_interval_ms: u64) {
    let mut interval = interval(Duration::from_millis(gc_interval_ms));
    while let Some(_interval) = interval.next().await {
        mempool
            .lock()
            .expect("[shared mempool] failed to acquire mempool lock")
            .gc();
    }

    error!("SharedMempool gc_task terminated");
}
