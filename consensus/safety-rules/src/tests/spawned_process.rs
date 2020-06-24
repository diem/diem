// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{process_client_wrapper::ProcessClientWrapper, tests::suite};
use libra_config::config::SecureBackend;

#[test]
fn test() {
    suite::run_test_suite(&safety_rules(false));
    suite::run_test_suite(&safety_rules(true));
}

fn safety_rules(verify_vote_proposal_signature: bool) -> suite::Callback {
    Box::new(move || {
        let mut client_wrapper = ProcessClientWrapper::new(
            SecureBackend::InMemoryStorage,
            verify_vote_proposal_signature,
        );
        let signer = client_wrapper.signer();
        let execution_private_key = if verify_vote_proposal_signature {
            Some(client_wrapper.execution_private_key())
        } else {
            None
        };
        (Box::new(client_wrapper), signer, execution_private_key)
    })
}
