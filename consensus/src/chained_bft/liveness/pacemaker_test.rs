// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::liveness::pacemaker::{
        ExponentialTimeInterval, NewRoundEvent, NewRoundReason, Pacemaker, PacemakerTimeInterval,
    },
    util::mock_time_service::SimulatedTimeService,
};

use channel;
use consensus_types::common::Round;
use futures::{executor::block_on, StreamExt};
use std::{sync::Arc, time::Duration};

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
/// Verify that Pacemaker properly outputs local timeout events upon timeout
fn test_basic_timeout() {
    let (mut pm, mut timeout_rx) = make_pacemaker();

    // jump start the pacemaker
    pm.process_certificates(Some(0), None, None);
    for _ in 0..2 {
        let round = block_on(timeout_rx.next()).unwrap();
        // Here we just test timeout send retry,
        // round for timeout is not changed as no timeout certificate was gathered at this point
        assert_eq!(1, round);
        pm.process_local_timeout(round);
    }
}

#[test]
fn test_round_event_generation() {
    let (mut pm, _) = make_pacemaker();
    // Happy path with new QC
    expect_qc(2, pm.process_certificates(Some(1), None, None));
    // Old QC does not generate anything
    assert!(pm.process_certificates(Some(1), None, None).is_none());
    // A TC for a higher round
    expect_timeout(3, pm.process_certificates(None, Some(2), None));
    // In case both QC and TC are present choose the one with the higher value
    expect_timeout(4, pm.process_certificates(Some(2), Some(3), None));
    // In case both QC and TC are present with the same value, choose QC
    expect_qc(5, pm.process_certificates(Some(4), Some(4), None));
}

fn make_pacemaker() -> (Pacemaker, channel::Receiver<Round>) {
    let time_interval = Box::new(ExponentialTimeInterval::fixed(Duration::from_millis(2)));
    let simulated_time = SimulatedTimeService::auto_advance_until(Duration::from_millis(4));
    let (timeout_tx, timeout_rx) = channel::new_test(1_024);
    (
        Pacemaker::new(time_interval, Arc::new(simulated_time), timeout_tx),
        timeout_rx,
    )
}

fn expect_qc(round: Round, event: Option<NewRoundEvent>) {
    let event = event.unwrap();
    assert_eq!(round, event.round);
    assert_eq!(event.reason, NewRoundReason::QCReady);
}

fn expect_timeout(round: Round, event: Option<NewRoundEvent>) {
    let event = event.unwrap();
    assert_eq!(round, event.round);
    assert_eq!(event.reason, NewRoundReason::Timeout);
}
