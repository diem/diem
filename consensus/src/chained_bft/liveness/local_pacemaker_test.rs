// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        liveness::{
            local_pacemaker::{ExponentialTimeInterval, LocalPacemaker, PacemakerTimeInterval},
            new_round_msg::PacemakerTimeout,
            pacemaker::{NewRoundEvent, NewRoundReason, PacemakerEvent},
            pacemaker_timeout_manager::HighestTimeoutCertificates,
        },
        persistent_storage::PersistentStorage,
        test_utils::{consensus_runtime, MockStorage, TestPayload},
    },
    mock_time_service::SimulatedTimeService,
    stream_utils::start_event_processing_loop,
};
use channel;
use futures::{channel::mpsc, executor::block_on, SinkExt, StreamExt};
use std::{sync::Arc, time::Duration, u64};
use tokio::runtime;
use types::validator_signer::ValidatorSigner;

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
/// Verify that LocalPacemaker properly outputs PacemakerTimeoutMsg upon timeout
fn test_basic_timeout() {
    let runtime = consensus_runtime();
    let time_interval = Box::new(ExponentialTimeInterval::fixed(Duration::from_millis(2)));
    let highest_certified_round = 1;
    let simulated_time = SimulatedTimeService::auto_advance_until(Duration::from_millis(4));
    let (pacemaker_timeout_tx, mut pacemaker_timeout_rx) = channel::new_test(1_024);
    let mut pm = Arc::new(LocalPacemaker::new(
        MockStorage::<TestPayload>::start_for_testing()
            .0
            .persistent_liveness_storage(),
        time_interval,
        0,
        highest_certified_round,
        Arc::new(simulated_time.clone()),
        pacemaker_timeout_tx,
        1,
        HighestTimeoutCertificates::new(None, None),
    ));

    start_event_processing_loop(&mut pm, runtime.executor());

    block_on(async move {
        for _ in 0..2 {
            let round = pacemaker_timeout_rx.next().await.unwrap();
            // Here we just test timeout send retry,
            // round for timeout is not changed as no timeout certificate was gathered at this point
            assert_eq!(2, round);
        }
    });
}

#[test]
/// Verify that LocalPacemaker forms a timeout certificate on receiving sufficient timeout messages
fn test_timeout_certificate() {
    let runtime = consensus_runtime();
    let rounds = 5;
    let mut signers: Vec<ValidatorSigner> = vec![];
    for _round in 1..rounds {
        let signer = ValidatorSigner::random();
        signers.push(signer);
    }
    let (mut tx, mut new_round_events_receiver) = make_pacemaker(&runtime);

    block_on(async move {
        // Send timeout for rounds 1..5, each from a different author, so that they can be
        // accumulated into single timeout certificate
        for round in 1..rounds {
            let signer = &signers[round - 1];
            let pacemaker_timeout = PacemakerTimeout::new(round as u64, signer);
            tx.send(PacemakerEvent::RemoteTimeout { pacemaker_timeout })
                .await
                .unwrap();
        }
        // First event sent automatically on pacemaker startup
        expect_qc(1, &mut new_round_events_receiver).await;
        // Then timeout quorum for previous round (1,2,3) generates new round event for round 2
        expect_timeout(2, &mut new_round_events_receiver).await;
        // Then timeout quorum for previous round (2,3,4) generates new round event for round 3
        expect_timeout(3, &mut new_round_events_receiver).await;
    });
}

#[test]
fn test_basic_qc() {
    let runtime = consensus_runtime();
    let (mut tx, mut new_round_intervals_receiver) = make_pacemaker(&runtime);

    block_on(async move {
        tx.send(PacemakerEvent::QuorumCertified { round: 2 })
            .await
            .unwrap();
        tx.send(PacemakerEvent::QuorumCertified { round: 3 })
            .await
            .unwrap();

        // The first event is just the initial round, the next two events are the new QCs.
        expect_qc(1, &mut new_round_intervals_receiver).await;
        expect_qc(3, &mut new_round_intervals_receiver).await;
        expect_qc(4, &mut new_round_intervals_receiver).await;
    });
}

fn make_pacemaker(
    runtime: &runtime::Runtime,
) -> (mpsc::Sender<PacemakerEvent>, mpsc::Receiver<NewRoundEvent>) {
    let time_interval = Box::new(ExponentialTimeInterval::fixed(Duration::from_millis(2)));
    let simulated_time = SimulatedTimeService::new();
    let (pacemaker_timeout_tx, _) = channel::new_test(1_024);
    let mut pm = Arc::new(LocalPacemaker::new(
        MockStorage::<TestPayload>::start_for_testing()
            .0
            .persistent_liveness_storage(),
        time_interval,
        0,
        0,
        Arc::new(simulated_time.clone()),
        pacemaker_timeout_tx,
        3,
        HighestTimeoutCertificates::new(None, None),
    ));
    start_event_processing_loop(&mut pm, runtime.executor())
}

async fn expect_qc(round: u64, rx: &mut mpsc::Receiver<NewRoundEvent>) {
    let event: NewRoundEvent = rx.next().await.unwrap();
    assert_eq!(round, event.round);
    assert_eq!(NewRoundReason::QCReady, event.reason);
}

async fn expect_timeout(round: u64, rx: &mut mpsc::Receiver<NewRoundEvent>) {
    let event: NewRoundEvent = rx.next().await.unwrap();
    assert_eq!(round, event.round);
    match event.reason {
        NewRoundReason::Timeout { .. } => (),
        x => panic!("Expected timeout for round {}, got {:?}", round, x),
    };
}
