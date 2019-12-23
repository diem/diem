// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{core_mempool::CoreMempool, shared_mempool::start_shared_mempool};
use admission_control_proto::proto::admission_control::{
    SubmitTransactionRequest, SubmitTransactionResponse,
};
use anyhow::Result;
use channel::libra_channel;
use channel::message_queues::QueueStyle;
use futures::channel::{
    mpsc::{self, unbounded},
    oneshot,
};
use libra_config::config::{NetworkConfig, NodeConfig};
use libra_types::PeerId;
use network::validator_network::{MempoolNetworkEvents, MempoolNetworkSender};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use storage_service::mocks::mock_storage_client::MockStorageReadClient;
use tokio::runtime::{Builder, Runtime};
use vm_validator::mocks::mock_vm_validator::MockVMValidator;

/// Creates a mock of a running instance of shared mempool
/// Returns the runtime on which the shared mempool is running
/// and the channel through which shared mempool receives client events
pub fn mock_shared_mempool() -> (
    Runtime,
    mpsc::Sender<(
        SubmitTransactionRequest,
        oneshot::Sender<Result<SubmitTransactionResponse>>,
    )>,
) {
    let runtime = Builder::new()
        .thread_name("shared-mem-")
        .threaded_scheduler()
        .enable_all()
        .build()
        .expect("[shared mempool] failed to create runtime");

    let peer_id = PeerId::random();
    let mut validator_network_config = NetworkConfig::default();
    validator_network_config.peer_id = peer_id;
    let mut config = NodeConfig::random();
    config.validator_network = Some(validator_network_config);

    let mempool = Arc::new(Mutex::new(CoreMempool::new(&config)));
    let (network_reqs_tx, _network_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let (_network_notifs_tx, network_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(8).unwrap(), None);
    let network_sender = MempoolNetworkSender::new(network_reqs_tx);
    let network_events = MempoolNetworkEvents::new(network_notifs_rx);
    let (sender, _subscriber) = unbounded();
    let (ac_sender, client_events) = mpsc::channel(1_024);
    let (_consensus_sender, consensus_events) = mpsc::channel(1_024);
    let network_handles = vec![(peer_id, network_sender, network_events)];

    start_shared_mempool(
        runtime.handle(),
        &config,
        mempool,
        network_handles,
        client_events,
        consensus_events,
        Arc::new(MockStorageReadClient),
        Arc::new(MockVMValidator),
        vec![sender],
        None,
    );

    (runtime, ac_sender)
}
