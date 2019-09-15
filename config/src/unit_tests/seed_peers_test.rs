// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::SeedPeersConfigHelpers;
use crate::trusted_peers::ConfigHelpers;

#[test]
fn generate_test_config() {
    let (_, consensus_peers_config) = ConfigHelpers::get_test_consensus_config(10, None);
    let (_, network_peers_config) =
        ConfigHelpers::get_test_network_peers_config(&consensus_peers_config, None);
    let _ = SeedPeersConfigHelpers::get_test_config(&network_peers_config, None);
}
