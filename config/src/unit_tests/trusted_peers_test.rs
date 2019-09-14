// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::ConfigHelpers;

#[test]
fn generate_test_config() {
    let (_keys, _consensus_peers_config, _network_peers_config) =
        ConfigHelpers::gen_validator_nodes(10, None);
    let (_keys, _network_peers_config) = ConfigHelpers::gen_full_nodes(10, None);
}
