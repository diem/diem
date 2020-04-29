// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::CoreMempool,
    network::{MempoolNetworkEvents, MempoolNetworkSender},
    shared_mempool::{
        coordinator::{broadcast_coordinator, gc_coordinator, request_coordinator},
        peer_manager::PeerManager,
        types::{
            IntervalStream, SharedMempool, SharedMempoolNotification,
            DEFAULT_MIN_BROADCAST_RECIPIENT_COUNT,
        },
    },
    CommitNotification, ConsensusRequest, SubmissionStatus,
};
use anyhow::Result;
use channel::{libra_channel, message_queues::QueueStyle};
use futures::channel::{
    mpsc::{self, Receiver, UnboundedSender},
    oneshot,
};
use libra_config::config::NodeConfig;
use libra_types::{on_chain_config::OnChainConfigPayload, transaction::SignedTransaction, PeerId};
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, Mutex, RwLock},
};
use storage_interface::DbReader;
use tokio::runtime::{Builder, Handle, Runtime};
use vm_validator::vm_validator::{TransactionValidation, VMValidator};

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
    mempool_reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
    db: Arc<dyn DbReader>,
    validator: Arc<RwLock<V>>,
    subscribers: Vec<UnboundedSender<SharedMempoolNotification>>,
    broadcast_ticker: Option<IntervalStream>,
) where
    V: TransactionValidation + 'static,
{
    let upstream_config = config.upstream.clone();
    let peer_manager = Arc::new(PeerManager::new(
        upstream_config,
        config
            .mempool
            .shared_mempool_min_broadcast_recipient_count
            .unwrap_or(DEFAULT_MIN_BROADCAST_RECIPIENT_COUNT),
    ));
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
        db,
        validator,
        peer_manager,
        subscribers,
    };

    let smp_outbound = smp.clone();
    let (schedule_broadcast_tx, schedule_broadcast_rx) = libra_channel::new(
        QueueStyle::LIFO,
        NonZeroUsize::new(1).expect("failed to set libra-channel size"),
        None,
    );
    executor.spawn(broadcast_coordinator(
        smp_outbound,
        schedule_broadcast_rx,
        executor.clone(),
        broadcast_ticker,
    ));

    executor.spawn(request_coordinator(
        smp,
        executor.clone(),
        all_network_events,
        client_events,
        consensus_requests,
        state_sync_requests,
        mempool_reconfig_events,
        schedule_broadcast_tx,
        config_clone,
    ));

    executor.spawn(gc_coordinator(
        mempool,
        config.mempool.system_transaction_gc_interval_ms,
    ));
}

/// method used to bootstrap shared mempool for a node
pub fn bootstrap(
    config: &NodeConfig,
    db: Arc<dyn DbReader>,
    // The first element in the tuple is the ID of the network that this network is a handle to
    // See `NodeConfig::is_upstream_peer` for the definition of network ID
    mempool_network_handles: Vec<(PeerId, MempoolNetworkSender, MempoolNetworkEvents)>,
    client_events: Receiver<(SignedTransaction, oneshot::Sender<Result<SubmissionStatus>>)>,
    consensus_requests: Receiver<ConsensusRequest>,
    state_sync_requests: Receiver<CommitNotification>,
    mempool_reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
) -> Runtime {
    let runtime = Builder::new()
        .thread_name("shared-mem-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[shared mempool] failed to create runtime");
    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
    let vm_validator = Arc::new(RwLock::new(VMValidator::new(Arc::clone(&db))));
    start_shared_mempool(
        runtime.handle(),
        config,
        mempool,
        mempool_network_handles,
        client_events,
        consensus_requests,
        state_sync_requests,
        mempool_reconfig_events,
        db,
        vm_validator,
        vec![],
        None,
    );
    runtime
}
