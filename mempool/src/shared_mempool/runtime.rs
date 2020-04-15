// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    core_mempool::CoreMempool,
    network::{MempoolNetworkEvents, MempoolNetworkSender},
    shared_mempool::{
        coordinator::{broadcast_coordinator, gc_coordinator, request_coordinator},
        types::{IntervalStream, PeerInfo, SharedMempool, SharedMempoolNotification, SyncEvent},
    },
    CommitNotification, ConsensusRequest, SubmissionStatus,
};
use anyhow::Result;
use channel::libra_channel;
use futures::{
    channel::{
        mpsc::{self, Receiver, UnboundedSender},
        oneshot,
    },
    StreamExt,
};
use libra_config::config::NodeConfig;
use libra_types::{on_chain_config::OnChainConfigPayload, transaction::SignedTransaction, PeerId};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use storage_client::{StorageRead, StorageReadServiceClient, SyncStorageClient};
use tokio::{
    runtime::{Builder, Handle, Runtime},
    sync::RwLock,
    time::interval,
};
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
    storage_read_client: Arc<dyn StorageRead>,
    validator: Arc<RwLock<V>>,
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
        broadcast_coordinator(smp_outbound, interval).await
    };

    executor.spawn(f);

    executor.spawn(request_coordinator(
        smp,
        executor.clone(),
        all_network_events,
        client_events,
        consensus_requests,
        state_sync_requests,
        mempool_reconfig_events,
        config_clone,
    ));

    executor.spawn(gc_coordinator(
        mempool,
        config.mempool.system_transaction_gc_interval_ms,
    ));
}

fn default_timer(tick_ms: u64) -> IntervalStream {
    interval(Duration::from_millis(tick_ms))
        .map(|_| SyncEvent)
        .boxed()
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
    mempool_reconfig_events: libra_channel::Receiver<(), OnChainConfigPayload>,
) -> Runtime {
    let runtime = Builder::new()
        .thread_name("shared-mem-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[shared mempool] failed to create runtime");
    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
    let storage_read_client: Arc<dyn StorageRead> =
        Arc::new(StorageReadServiceClient::new(&config.storage.address));
    let db_reader = Arc::new(SyncStorageClient::new(&config.storage.address));
    let vm_validator = Arc::new(RwLock::new(VMValidator::new(db_reader)));
    start_shared_mempool(
        runtime.handle(),
        config,
        mempool,
        mempool_network_handles,
        client_events,
        consensus_requests,
        state_sync_requests,
        mempool_reconfig_events,
        storage_read_client,
        vm_validator,
        vec![],
        None,
    );
    runtime
}
