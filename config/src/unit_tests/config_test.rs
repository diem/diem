// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;

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

    let actual_network = actual
        .validator_network
        .as_mut()
        .expect("Missing actual network config");
    let expected_network = expected
        .validator_network
        .as_mut()
        .expect("Missing expected network config");

    expected_network.advertised_address = actual_network.advertised_address.clone();
    expected_network.listen_address = actual_network.listen_address.clone();
    expected_network.network_keypairs = actual_network.network_keypairs.clone();
    expected_network.network_peers = actual_network.network_peers.clone();
    expected_network.seed_peers = actual_network.seed_peers.clone();

    // This is broken down first into smaller evaluations to improve idenitfying what is broken.
    // The output for a broken config leveraging assert at the top level config is not readable.
    assert_eq!(actual.admission_control, expected.admission_control);
    assert_eq!(actual.base, expected.base);
    assert_eq!(actual.consensus, expected.consensus);
    assert_eq!(actual.debug_interface, expected.debug_interface);
    assert_eq!(actual.execution, expected.execution);
    assert_eq!(actual.full_node_networks, expected.full_node_networks);
    assert_eq!(actual.full_node_networks.len(), 0);
    assert_eq!(actual.logger, expected.logger);
    assert_eq!(actual.mempool, expected.mempool);
    assert_eq!(actual.metrics, expected.metrics);
    assert_eq!(actual.state_sync, expected.state_sync);
    assert_eq!(actual.storage, expected.storage);
    assert_eq!(actual.validator_network, expected.validator_network);
    assert_eq!(actual.vm_config, expected.vm_config);
    assert_eq!(actual, expected);
}

#[test]
fn verify_all_configs() {
    let _ = vec![
        PathBuf::from("data/configs/overrides/persistent_data.node.config.override.toml"),
        PathBuf::from("data/configs/full_node.config.toml"),
        PathBuf::from("data/configs/node.config.toml"),
        PathBuf::from("data/configs/single.node.config.toml"),
        PathBuf::from("../terraform/validator-sets/100/fn/node.config.toml"),
        PathBuf::from("../terraform/validator-sets/100/val/node.config.toml"),
        PathBuf::from("../terraform/validator-sets/dev/fn/node.config.toml"),
        PathBuf::from("../terraform/validator-sets/dev/val/node.config.toml"),
    ]
    .iter()
    .map(|path| NodeConfig::load(path).expect("NodeConfig"));
}
