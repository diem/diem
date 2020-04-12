// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, SafetyRulesManager};
use consensus_types::common::Round;
use libra_types::validator_signer::ValidatorSigner;

#[test]
fn test_reconnect() {
    let signer = ValidatorSigner::from_int(0);
    let storage = test_utils::test_storage(&signer);
    let safety_rules_manager = SafetyRulesManager::<Round>::new_thread(signer.author(), storage);

    // Verify that after a client has disconnected a new client will connect and resume operations
    let state0 = safety_rules_manager.client().consensus_state().unwrap();
    let state1 = safety_rules_manager.client().consensus_state().unwrap();
    assert_eq!(state0, state1);
}
