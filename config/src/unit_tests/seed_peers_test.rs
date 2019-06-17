// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::SeedPeersConfigHelpers;
use crate::trusted_peers::TrustedPeersConfigHelpers;

#[test]
fn generate_test_config() {
    let (_, trusted_peers) = TrustedPeersConfigHelpers::get_test_config(10, None);
    let _ = SeedPeersConfigHelpers::get_test_config(&trusted_peers, None);
}
