// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    consensus_types::timeout_msg::{PacemakerTimeout, PacemakerTimeoutCertificate},
    liveness::pacemaker_timeout_manager::{HighestTimeoutCertificates, PacemakerTimeoutManager},
    persistent_storage::PersistentStorage,
    test_utils::{MockStorage, TestPayload},
};
use types::validator_signer::ValidatorSigner;

#[test]
fn test_basic() {
    let mut timeout_manager = PacemakerTimeoutManager::new(
        2,
        HighestTimeoutCertificates::new(None, None),
        MockStorage::<TestPayload>::start_for_testing()
            .0
            .persistent_liveness_storage(),
    );
    assert_eq!(timeout_manager.highest_timeout_certificate(), None);
    let validator_signer1 = ValidatorSigner::random([0u8; 32]);
    let validator_signer2 = ValidatorSigner::random([1u8; 32]);

    // No timeout certificate generated on adding 2 timeouts from the same author
    let timeout_signer1_round1 = PacemakerTimeout::new(1, &validator_signer1);
    assert_eq!(
        timeout_manager.update_received_timeout(timeout_signer1_round1),
        false
    );
    assert_eq!(timeout_manager.highest_timeout_certificate(), None);
    let timeout_signer1_round2 = PacemakerTimeout::new(2, &validator_signer1);
    assert_eq!(
        timeout_manager.update_received_timeout(timeout_signer1_round2),
        false
    );
    assert_eq!(timeout_manager.highest_timeout_certificate(), None);

    // Timeout certificate generated on adding a timeout from signer2
    let timeout_signer2_round1 = PacemakerTimeout::new(1, &validator_signer2);
    assert_eq!(
        timeout_manager.update_received_timeout(timeout_signer2_round1),
        true
    );
    assert_eq!(
        timeout_manager
            .highest_timeout_certificate()
            .unwrap()
            .round(),
        1
    );

    // Timeout certificate increased when incrementing the round from signer 2
    let timeout_signer2_round2 = PacemakerTimeout::new(2, &validator_signer2);
    assert_eq!(
        timeout_manager.update_received_timeout(timeout_signer2_round2),
        true
    );
    assert_eq!(
        timeout_manager
            .highest_timeout_certificate()
            .unwrap()
            .round(),
        2
    );

    // No timeout certificate generated since signer 1 is still on round 2
    let timeout_signer2_round3 = PacemakerTimeout::new(3, &validator_signer2);
    assert_eq!(
        timeout_manager.update_received_timeout(timeout_signer2_round3),
        false
    );
    assert_eq!(
        timeout_manager
            .highest_timeout_certificate()
            .unwrap()
            .round(),
        2
    );

    // Simulate received a higher received timeout certificate
    let received_timeout_certificate = PacemakerTimeoutCertificate::new(
        10,
        vec![
            PacemakerTimeout::new(10, &validator_signer1),
            PacemakerTimeout::new(11, &validator_signer2),
        ],
    );
    assert_eq!(
        timeout_manager.update_highest_received_timeout_certificate(&received_timeout_certificate),
        true
    );
    assert_eq!(
        timeout_manager
            .highest_timeout_certificate()
            .unwrap()
            .round(),
        10
    );
}

#[test]
fn test_recovery_from_highest_timeout_certificate() {
    let validator_signer1 = ValidatorSigner::random([0u8; 32]);
    let validator_signer2 = ValidatorSigner::random([1u8; 32]);

    let timeout1 = PacemakerTimeout::new(10, &validator_signer1);
    let timeout2 = PacemakerTimeout::new(11, &validator_signer2);
    let tc = PacemakerTimeoutCertificate::new(10, vec![timeout1, timeout2]);

    let timeout_manager = PacemakerTimeoutManager::new(
        2,
        HighestTimeoutCertificates::new(Some(tc), None),
        MockStorage::<TestPayload>::start_for_testing()
            .0
            .persistent_liveness_storage(),
    );

    assert_eq!(
        timeout_manager
            .author_to_received_timeouts
            .contains_key(&validator_signer1.author()),
        true
    );
    assert_eq!(
        timeout_manager
            .author_to_received_timeouts
            .contains_key(&validator_signer2.author()),
        true
    );
}
