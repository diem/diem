// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::smoke_test_environment::SmokeTestEnvironment;
use diem_config::{
    config::{DiscoveryMethod, NodeConfig, PeerSet, PersistableConfig},
    network_id::NetworkId,
};
use diem_crypto::x25519;
use diem_operational_tool::{
    keys::{EncodingType, KeyType},
    test_helper::OperationalTool,
};
use diem_swarm::swarm::{network_mut, DiemNode};
use diem_temppath::TempPath;
use std::{collections::HashMap, path::Path, thread::sleep, time::Duration};

#[test]
fn test_file_discovery() {
    let mut env = SmokeTestEnvironment::new(1);
    // Add file for discovery with nothing in it
    let discovery_file = TempPath::new();
    discovery_file.create_as_file().unwrap();
    let op_tool = OperationalTool::test();
    let key_file = TempPath::new();
    key_file.create_as_file().unwrap();
    let private_key = op_tool
        .generate_key(KeyType::X25519, key_file.as_ref(), EncodingType::BCS)
        .unwrap();
    // When we add the private key, we should be able to connect
    let peer_set = op_tool
        .extract_peer_from_file(key_file.as_ref(), EncodingType::BCS)
        .unwrap();
    write_peerset_to_file(discovery_file.as_ref(), peer_set);

    // Update to use the file based discovery with onchain
    let node_config_path = env.validator_swarm.config.config_files.get(0).unwrap();
    let mut node_config = NodeConfig::load(&node_config_path).unwrap();
    let network = network_mut(&mut node_config, &NetworkId::Validator);
    network.discovery_method = DiscoveryMethod::None;
    network.discovery_methods = vec![
        DiscoveryMethod::Onchain,
        DiscoveryMethod::File(
            discovery_file.as_ref().to_path_buf(),
            Duration::from_secs(1),
        ),
    ];
    node_config.save_config(node_config_path).unwrap();

    // Startup the validator
    env.validator_swarm.launch();

    // At first we shouldn't be able to connect
    assert_eq!(
        true,
        check_endpoint(
            &op_tool,
            NetworkId::Validator,
            env.validator_swarm.get_node(0).unwrap(),
            &private_key
        )
    );

    // Here we have to wait, todo fix this
    write_peerset_to_file(discovery_file.as_ref(), HashMap::new());
    sleep(Duration::from_secs(3));

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
