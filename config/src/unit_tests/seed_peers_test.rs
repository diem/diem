// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::SeedPeersConfigHelpers;
use crate::trusted_peers::ConfigHelpers;

#[test]
fn generate_test_config() {
    let (_, _, network_peers_config) = ConfigHelpers::gen_validator_nodes(10, None);
    let _ = SeedPeersConfigHelpers::get_test_config(&network_peers_config, None);
}
