// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use lazy_static::lazy_static;

// A list of metrics which will be made public to all the partners
lazy_static! {
    pub static ref PUBLIC_METRICS: Vec<String> =
        vec!["libra_network_peers".to_string(), "revision".to_string()];
}
