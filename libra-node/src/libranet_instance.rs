// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::NodeConfig;
use network_builder::builder::NetworkBuilder;
use tokio::runtime::{Builder, Runtime};
use libra_types::chain_id::ChainId;
use libra_types::PeerId;
use libra_network_address::NetworkAddress;
use std::collections::{HashMap, HashSet};
use libra_crypto::x25519;
use futures::{sink::SinkExt};
use network::ConnectivityRequest;
use network::connectivity_manager::DiscoverySource;
use libra_mempool::network::{MempoolNetworkSender, MempoolNetworkEvents};
use state_synchronizer::network::{StateSynchronizerSender, StateSynchronizerEvents};

pub struct PublicLibraNetInstance {
    _network_runtime: Vec<Runtime>,
    pub mempool_handles: Vec<(MempoolNetworkSender, MempoolNetworkEvents)>,
    pub state_sync_handles: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>
}

pub async fn launch(chain_id: ChainId, seed_peers: HashMap<PeerId, Vec<NetworkAddress>>) -> PublicLibraNetInstance {
    let mut network_runtimes = vec![];
    let mut network_builders = vec![];
    let mut mempool_handles = vec![];
    let mut state_sync_handles = vec![];

    let pfn_config = NodeConfig::default_for_public_full_node();
    for network_config in pfn_config.full_node_networks.iter() {
        let mut network_builder = NetworkBuilder::create(chain_id, pfn_config.base.role, network_config);

        let (state_sync_sender, state_sync_events) = network_builder
            .add_protocol_handler(state_synchronizer::network::network_endpoint_config());

        state_sync_handles.push((state_sync_sender, state_sync_events));

        let (mempool_sender, mempool_events) = network_builder
            .add_protocol_handler(libra_mempool::network::network_endpoint_config(pfn_config.mempool.max_broadcasts_per_peer));
        mempool_handles.push((mempool_sender, mempool_events));

        network_builders.push(network_builder);
    }

    for network_builder in &mut network_builders {
        let runtime = Builder::new()
            .thread_name("network-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("Failed to start runtime. Won't be able to start networking.");

        network_builder.build(runtime.handle().clone());
        network_runtimes.push(runtime);
    }

    for network_builder in &mut network_builders {
        network_builder.start();
    }

    // feed the network builder the seed peer config
    let mut conn_req_tx = network_builders[0].conn_mgr_reqs_tx().expect("expecting connectivity mgr to exist after adding protocol handler");

    let new_peer_pubkeys: HashMap<_, _> = seed_peers
        .iter()
        .map(|(peer_id, addrs)| {
            let pubkeys: HashSet<x25519::PublicKey> = addrs
                .iter()
                .filter_map(NetworkAddress::find_noise_proto)
                .collect();
            (*peer_id, pubkeys)
        })
        .collect();

    let conn_reqs = vec![
        ConnectivityRequest::UpdateAddresses(DiscoverySource::OnChain, seed_peers),
        ConnectivityRequest::UpdateEligibleNodes(DiscoverySource::OnChain, new_peer_pubkeys),
    ];

    for update in conn_reqs {
        conn_req_tx.send(update).await.expect("failed to send conn req");
    }

    PublicLibraNetInstance {
        _network_runtime: network_runtimes,
        mempool_handles,
        state_sync_handles,
    }

}