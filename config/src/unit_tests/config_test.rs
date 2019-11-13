// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::fs;

static EXPECTED_SINGLE_NODE_CONFIG: &[u8] =
    include_bytes!("../../data/configs/single.node.config.toml");

#[test]
fn verify_test_config() {
    // This test likely failed because there was a breaking change in the NodeConfig. It may be
    // desirable to reverse the change or to change the test config and potentially documentation.
    let mut actual = NodeConfigHelpers::get_single_node_test_config(false);
    let mut expected = NodeConfig::parse(&String::from_utf8_lossy(EXPECTED_SINGLE_NODE_CONFIG))
        .expect("Error parsing expected single node config");

    // These are randomly generated, so let's force them to be the same, perhaps we can use a
    // random seed so that these can be made uniform...
    expected.base.data_dir_path = actual.base.data_dir_path.clone();
    expected.base.temp_data_dir = None;
    actual.base.temp_data_dir = None;
    expected.consensus.consensus_keypair = actual.consensus.consensus_keypair.clone();
    expected.consensus.consensus_peers = actual.consensus.consensus_peers.clone();

    assert_eq!(actual.networks.len(), expected.networks.len());
    for x in 0..actual.networks.len() {
        expected.networks[x].advertised_address = actual.networks[x].advertised_address.clone();
        expected.networks[x].listen_address = actual.networks[x].listen_address.clone();
        expected.networks[x].network_keypairs = actual.networks[x].network_keypairs.clone();
        expected.networks[x].network_peers = actual.networks[x].network_peers.clone();
        expected.networks[x].seed_peers = actual.networks[x].seed_peers.clone();
    }

    // This is broken down first into smaller evaluations to improve idenitfying what is broken.
    // The output for a broken config leveraging assert at the top level config is not readable.
    assert_eq!(actual.base, expected.base);
    assert_eq!(actual.metrics, expected.metrics);
    assert_eq!(actual.execution, expected.execution);
    assert_eq!(actual.admission_control, expected.admission_control);
    assert_eq!(actual.debug_interface, expected.debug_interface);
    assert_eq!(actual.storage, expected.storage);
    assert_eq!(actual.networks, expected.networks);
    assert_eq!(actual.consensus, expected.consensus);
    assert_eq!(actual.mempool, expected.mempool);
    assert_eq!(actual.state_sync, expected.state_sync);
    assert_eq!(actual.logger, expected.logger);
    assert_eq!(actual.vm_config, expected.vm_config);
    assert_eq!(actual, expected);
}

#[test]
fn verify_all_configs() {
    // This test verifies that all configs in data/config are valid
    let paths = fs::read_dir("data/configs").expect("cannot read config dir");

    for path in paths {
        let config_path = path.unwrap().path();
        let config_path_str = config_path.to_str().unwrap();
        if config_path_str.ends_with(".toml") {
            println!("Loading {}", config_path_str);
            let _ = NodeConfig::load(config_path_str).expect("NodeConfig");
        } else {
            println!("Invalid file {} for verifying", config_path_str);
        }
    }
}
