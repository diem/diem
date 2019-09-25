// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::Round,
        consensus_types::timeout_msg::PacemakerTimeout,
        liveness::{
            pacemaker::{
                ExponentialTimeInterval, NewRoundEvent, NewRoundReason, Pacemaker,
                PacemakerTimeInterval,
            },
            pacemaker_timeout_manager::HighestTimeoutCertificates,
        },
        persistent_storage::PersistentStorage,
        test_utils::{MockStorage, TestPayload},
    },
    util::mock_time_service::SimulatedTimeService,
};
use channel;
use futures::{executor::block_on, StreamExt};
use std::{sync::Arc, time::Duration, u64};
use types::crypto_proxies::random_validator_verifier;

#[test]
fn test_pacemaker_time_interval() {
    let interval = ExponentialTimeInterval::new(Duration::from_millis(3000), 1.5, 2);
    assert_eq!(3000, interval.get_round_duration(0).as_millis());
    assert_eq!(4500, interval.get_round_duration(1).as_millis());
    assert_eq!(
        6750, /* 4500*1.5 */
        interval.get_round_duration(2).as_millis()
    );
    // Test that there is no integer overflow
    assert_eq!(6750, interval.get_round_duration(1000).as_millis());
}

#[test]
/// Verify that Pacemaker properly outputs PacemakerTimeoutMsg upon timeout
fn test_basic_timeout() {
    let (mut pm, mut timeout_rx) = make_pacemaker();

    // jump start the pacemaker
    pm.process_certificates(1, None, None);
    for _ in 0..2 {
        let round = block_on(timeout_rx.next()).unwrap();
        // Here we just test timeout send retry,
        // round for timeout is not changed as no timeout certificate was gathered at this point
        assert_eq!(2, round);
        pm.process_local_timeout(round);
    }
}

#[test]
/// Verify that Pacemaker forms a timeout certificate on receiving sufficient timeout messages
fn test_timeout_certificate() {
    let rounds: Round = 5;
    let (signers, validator_verifier) =
        random_validator_verifier((rounds - 1) as usize, None, false);
    let validator_verifier = Arc::new(validator_verifier);
    let (mut pm, _) = make_pacemaker();

    // Send timeout for rounds 1..5, each from a different author, so that they can be
    // accumulated into single timeout certificate
    for round in 1..rounds {
        let signer = &signers[(round - 1) as usize];
        let pacemaker_timeout = PacemakerTimeout::new(round, signer, None);
        let result = pm.process_remote_timeout(pacemaker_timeout, Arc::clone(&validator_verifier));
        // quorum size is 3 in make_pacemaker
        if round >= 3 {
            // Then timeout quorum for previous round (1,2,3) generates new round event for
            // round 2, timeout quorum for previous round (2,3,4) generates
            // new round event for round 3
            expect_timeout(round - 1, result);
        }
    }
}

#[test]
fn test_basic_qc() {
    let (mut pm, _) = make_pacemaker();

    expect_qc(3, pm.process_certificates(2, None, None));
    expect_qc(4, pm.process_certificates(3, None, None));
}

fn make_pacemaker() -> (Pacemaker, channel::Receiver<Round>) {
    let time_interval = Box::new(ExponentialTimeInterval::fixed(Duration::from_millis(2)));
    let simulated_time = SimulatedTimeService::auto_advance_until(Duration::from_millis(4));
    let (timeout_tx, timeout_rx) = channel::new_test(1_024);
    (
        Pacemaker::new(
            MockStorage::<TestPayload>::start_for_testing()
                .0
                .persistent_liveness_storage(),
            time_interval,
            Arc::new(simulated_time.clone()),
            timeout_tx,
            HighestTimeoutCertificates::default(),
        ),
        timeout_rx,
    )
}

fn expect_qc(round: u64, event: Option<NewRoundEvent>) {
    let event = event.unwrap();
    assert_eq!(round, event.round);
    assert_eq!(NewRoundReason::QCReady, event.reason);
}

fn expect_timeout(round: u64, event: Option<NewRoundEvent>) {
    let event = event.unwrap();
    assert_eq!(round, event.round);
    match event.reason {
        NewRoundReason::Timeout { .. } => (),
        x => panic!("Expected timeout for round {}, got {:?}", round, x),
    };
}
