// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::ConfigHelpers;

#[test]
fn generate_test_config() {
    let (_, consensus_peers_config) = ConfigHelpers::get_test_consensus_config(10, None);
    let (_, _) = ConfigHelpers::get_test_network_peers_config(&consensus_peers_config, None);
}
