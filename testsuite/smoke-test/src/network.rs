// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::smoke_test_environment::SmokeTestEnvironment;
use diem_config::{
    config::{DiscoveryMethod, Identity, NetworkConfig, NodeConfig, PeerSet, PersistableConfig},
    network_id::NetworkId,
};
use diem_crypto::{x25519, x25519::PrivateKey};
use diem_operational_tool::{
    keys::{EncodingType, KeyType},
    test_helper::OperationalTool,
};
use diem_swarm::swarm::{modify_network_config, DiemNode, DiemSwarm};
use diem_temppath::TempPath;
use std::{collections::HashMap, path::Path, thread::sleep, time::Duration};

#[test]
fn test_connection_limiting() {
    const SUCCESS: &str = "success";
    const FAIL: &str = "fail";

    let mut env = SmokeTestEnvironment::new(1);
    env.setup_vfn_swarm();

    let op_tool = OperationalTool::test();
    let (private_key, peer_set) = generate_private_key_and_peer(&op_tool);
    let discovery_file = create_discovery_file(peer_set.clone());

    // Only allow file based discovery, disallow other nodes
    modify_network_of_node(&env.vfn_swarm().lock(), 0, &NetworkId::Public, |network| {
        network.discovery_method = DiscoveryMethod::None;
        network.discovery_methods = vec![
            DiscoveryMethod::Onchain,
            DiscoveryMethod::File(
                discovery_file.as_ref().to_path_buf(),
                Duration::from_secs(1),
            ),
        ];
        network.max_inbound_connections = 0;
    });

    // Startup the validator & vfn
    env.validator_swarm.launch();
    env.vfn_swarm().lock().launch();

    // This node should be able to connect
    env.add_public_fn_swarm(SUCCESS, 1, None, &env.vfn_swarm().lock().config);
    add_identity_to_node(
        &env.public_swarm(SUCCESS).lock(),
        0,
        &NetworkId::Public,
        private_key,
        peer_set,
    );
    // This node should connect
    env.public_swarm(SUCCESS).lock().launch();
    assert_eq!(
        1,
        inbound_connected_peers(&mut env.vfn_swarm().lock(), 0, NetworkId::Public)
    );

    // And not be able to connect with an arbitrary one, limit is 0
    // TODO: Improve network checker to keep connection alive so we can test connection limits without nodes
    let (private_key, peer_set) = generate_private_key_and_peer(&op_tool);
    env.add_public_fn_swarm(FAIL, 1, None, &env.vfn_swarm().lock().config);
    add_identity_to_node(
        &env.public_swarm(FAIL).lock(),
        0,
        &NetworkId::Public,
        private_key,
        peer_set,
    );

    // This node should fail to connect
    env.public_swarm(FAIL).lock().launch_attempt(false).unwrap();
    sleep(Duration::from_secs(1));
    assert_eq!(
        1,
        inbound_connected_peers(&mut env.vfn_swarm().lock(), 0, NetworkId::Public)
    );
}

#[test]
fn test_file_discovery() {
    let mut env = SmokeTestEnvironment::new(1);
    let op_tool = OperationalTool::test();
    let (private_key, peer_set) = generate_private_key_and_peer(&op_tool);
    let discovery_file = create_discovery_file(peer_set);

    // Add key to file based discovery
    modify_network_of_node(&env.validator_swarm, 0, &NetworkId::Validator, |network| {
        network.discovery_method = DiscoveryMethod::None;
        network.discovery_methods = vec![
            DiscoveryMethod::Onchain,
            DiscoveryMethod::File(
                discovery_file.as_ref().to_path_buf(),
                Duration::from_millis(100),
            ),
        ];
    });

    // Startup the validator
    env.validator_swarm.launch();

    // At first we should be able to connect
    assert_eq!(
        true,
        check_endpoint(
            &op_tool,
            NetworkId::Validator,
            env.validator_swarm.get_node(0).unwrap(),
            &private_key
        )
    );

    // Now when we clear the file, we shouldn't be able to connect
    write_peerset_to_file(discovery_file.as_ref(), HashMap::new());
    sleep(Duration::from_millis(300));

    assert_eq!(
        false,
        check_endpoint(
            &op_tool,
            NetworkId::Validator,
            env.validator_swarm.get_node(0).unwrap(),
            &private_key
        )
    );
}

/// Creates a discovery file with the given `PeerSet`
fn create_discovery_file(peer_set: PeerSet) -> TempPath {
    let discovery_file = TempPath::new();
    discovery_file.create_as_file().unwrap();
    write_peerset_to_file(discovery_file.as_ref(), peer_set);
    discovery_file
}

/// Generates `PrivateKey` and `Peer` information for a client / node
fn generate_private_key_and_peer(op_tool: &OperationalTool) -> (PrivateKey, PeerSet) {
    let key_file = TempPath::new();
    key_file.create_as_file().unwrap();
    let private_key = op_tool
        .generate_key(KeyType::X25519, key_file.as_ref(), EncodingType::BCS)
        .unwrap();
    let peer_set = op_tool
        .extract_peer_from_file(key_file.as_ref(), EncodingType::BCS)
        .unwrap();
    (private_key, peer_set)
}

/// Modifies a network on the on disk configuration.  Needs to be done prior to starting node
fn modify_network_of_node<F: FnOnce(&mut NetworkConfig)>(
    swarm: &DiemSwarm,
    node_index: usize,
    network_id: &NetworkId,
    modifier: F,
) {
    let node_config_path = swarm.config.config_files.get(node_index).unwrap();
    let mut node_config = NodeConfig::load(&node_config_path).unwrap();
    modify_network_config(&mut node_config, network_id, modifier);
    node_config.save_config(node_config_path).unwrap();
}

fn add_identity_to_node(
    swarm: &DiemSwarm,
    node_index: usize,
    network_id: &NetworkId,
    private_key: PrivateKey,
    peer_set: PeerSet,
) {
    let (peer_id, _) = peer_set.iter().next().unwrap();
    modify_network_of_node(swarm, node_index, network_id, |network| {
        network.identity = Identity::from_config(private_key, *peer_id);
    });
}

fn inbound_connected_peers(swarm: &mut DiemSwarm, node_index: usize, network_id: NetworkId) -> i64 {
    swarm
        .mut_node(node_index)
        .unwrap()
        .get_connected_peers(network_id, Some("inbound"))
        .unwrap_or(0)
}

fn check_endpoint(
    op_tool: &OperationalTool,
    network_id: NetworkId,
    node: &DiemNode,
    private_key: &x25519::PrivateKey,
) -> bool {
    let address = node.network_address(&network_id);
    let result = op_tool.check_endpoint_with_key(&network_id, address.clone(), &private_key);
    println!(
        "Endpoint check for {}:{} is:  {:?}",
        network_id, address, result
    );
    result.is_ok()
}

pub fn write_peerset_to_file(path: &Path, peers: PeerSet) {
    let file_contents = serde_yaml::to_vec(&peers).unwrap();
    std::fs::write(path, file_contents).unwrap();
}
