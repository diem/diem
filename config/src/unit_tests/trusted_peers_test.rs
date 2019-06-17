// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::TrustedPeersConfigHelpers;

#[test]
fn generate_test_config() {
    let (_, _) = TrustedPeersConfigHelpers::get_test_config(10, None);
}
