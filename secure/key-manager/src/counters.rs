// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_secure_push_metrics::{define_counters, Counter};
use once_cell::sync::Lazy;
use std::sync::Arc;

// Metrics for the key manager. Uses the "libra_key_manager" prefix for all counters.
// TODO(joshlind): look into managing these metrics according to key type (i.e., when the
// metrics crate we're using supports labels).
define_counters![
    "libra_key_manager",
    (
        completed_consensus_key_rotations: Counter,
        "counts the number of completed consensus key rotations performed by the key manager"
    ),
    (
        consensus_rotation_tx_resubmissions: Counter,
        "counts the number of times the key manager had to resubmit a consensus rotation transaction to the blockchain"
    ),
    (
        no_actions_required: Counter,
        "counts the number of times the key manager determined that no actions were required"
    ),
    (
        waiting_on_consensus_reconfiguration: Counter,
        "counts the number of times the key manager had to wait for a reconfiguration event for the consensus key"
    ),
    (
        sleeps: Counter,
        "counts the number of times the key manager went to sleep"
    ),
];

pub static COUNTERS: Lazy<Arc<Counters>> = Lazy::new(|| Arc::new(Counters::new()));
