// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters,
    util::time_service::{SendTask, TimeService},
};
use channel;
use consensus_types::common::Round;
use libra_logger::prelude::*;
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

/// A reason for starting a new round: introduced for monitoring / debug purposes.
#[derive(Eq, Debug, PartialEq)]
pub enum NewRoundReason {
    QCReady,
    Timeout,
}

impl fmt::Display for NewRoundReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NewRoundReason::QCReady => write!(f, "QCReady"),
            NewRoundReason::Timeout => write!(f, "TCReady"),
        }
    }
}

/// NewRoundEvents produced by Pacemaker are guaranteed to be monotonically increasing.
/// NewRoundEvents are consumed by the rest of the system: they can cause sending new proposals
/// or voting for some proposals that wouldn't have been voted otherwise.
/// The duration is populated for debugging and testing
#[derive(Debug, PartialEq, Eq)]
pub struct NewRoundEvent {
    pub round: Round,
    pub reason: NewRoundReason,
    pub timeout: Duration,
}

impl fmt::Display for NewRoundEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "NewRoundEvent: [round: {}, reason: {}, timeout: {:?}]",
            self.round, self.reason, self.timeout
        )
    }
}

/// Determines the maximum round duration based on the round difference between the current
/// round and the committed round
pub trait PacemakerTimeInterval: Send + Sync + 'static {
    /// Use the index of the round after the highest quorum certificate to commit a block and
    /// return the duration for this round
    ///
    /// Round indices start at 0 (round index = 0 is the first round after the round that led
    /// to the highest committed round).  Given that round r is the highest round to commit a
    /// block, then round index 0 is round r+1.  Note that for genesis does not follow the
    /// 3-chain rule for commits, so round 1 has round index 0.  For example, if one wants
    /// to calculate the round duration of round 6 and the highest committed round is 3 (meaning
    /// the highest round to commit a block is round 5, then the round index is 0.
    fn get_round_duration(&self, round_index_after_committed_qc: usize) -> Duration;
}

/// Round durations increase exponentially
/// Basically time interval is base * mul^power
/// Where power=max(rounds_since_qc, max_exponent)
#[derive(Clone)]
pub struct ExponentialTimeInterval {
    // Initial time interval duration after a successful quorum commit.
    base_ms: u64,
    // By how much we increase interval every time
    exponent_base: f64,
    // Maximum time interval won't exceed base * mul^max_pow.
    // Theoretically, setting it means
    // that we rely on synchrony assumptions when the known max messaging delay is
    // max_interval.  Alternatively, we can consider using max_interval to meet partial synchrony
    // assumptions where while delta is unknown, it is <= max_interval.
    max_exponent: usize,
}

#[allow(dead_code)]
impl ExponentialTimeInterval {
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn fixed(duration: Duration) -> Self {
        Self::new(duration, 1.0, 0)
    }

    pub fn new(base: Duration, exponent_base: f64, max_exponent: usize) -> Self {
        assert!(
            max_exponent < 32,
            "max_exponent for PacemakerTimeInterval should be <32"
        );
        assert!(
            exponent_base.powf(max_exponent as f64).ceil() < f64::from(std::u32::MAX),
            "Maximum interval multiplier should be less then u32::Max"
        );
        ExponentialTimeInterval {
            base_ms: base.as_millis() as u64, // any reasonable ms timeout fits u64 perfectly
            exponent_base,
            max_exponent,
        }
    }
}

impl PacemakerTimeInterval for ExponentialTimeInterval {
    fn get_round_duration(&self, round_index_after_committed_qc: usize) -> Duration {
        let pow = round_index_after_committed_qc.min(self.max_exponent) as u32;
        let base_multiplier = self.exponent_base.powf(f64::from(pow));
        let duration_ms = ((self.base_ms as f64) * base_multiplier).ceil() as u64;
        Duration::from_millis(duration_ms)
    }
}

/// `Pacemaker` is a Pacemaker implementation that is responsible for generating the new round
/// and local timeout events.
///
/// A round `r` starts in the following cases:
/// * there is a QuorumCert for round `r-1`,
/// * there is a TimeoutCertificate for round `r-1`.
///
/// Round interval calculation is the responsibility of the PacemakerTimeoutInterval trait. It
/// depends on the delta between the current round and the highest committed round (the intuition is
/// that we want to exponentially grow the interval the further the current round is from the last
/// committed round).
///
/// Whenever a new round starts a local timeout is set following the round interval. This local
/// timeout is going to send the timeout events once in interval until the new round starts.
pub struct Pacemaker {
    // Determines the time interval for a round given the number of non-committed rounds since
    // last commit.
    time_interval: Box<dyn PacemakerTimeInterval>,
    // Highest known committed round as reported by the caller. The caller might choose not to
    // inform the Pacemaker about certain committed rounds (e.g., NIL blocks): in this case the
    // committed round in Pacemaker might lag behind the committed round of a block tree.
    highest_committed_round: Round,
    // Current round is max{highest_qc, highest_tc} + 1.
    current_round: Round,
    // The deadline for the next local timeout event. It is reset every time a new round start, or
    // a previous deadline expires.
    current_round_deadline: Instant,
    // Service for timer
    time_service: Arc<dyn TimeService>,
    // To send local timeout events to the subscriber (e.g., SMR)
    timeout_sender: channel::Sender<Round>,
}

#[allow(dead_code)]
impl Pacemaker {
    pub fn new(
        time_interval: Box<dyn PacemakerTimeInterval>,
        time_service: Arc<dyn TimeService>,
        timeout_sender: channel::Sender<Round>,
    ) -> Self {
        // Our counters are initialized via lazy_static, so they're not going to appear in
        // Prometheus if some conditions never happen. Invoking get() function enforces creation.
        counters::QC_ROUNDS_COUNT.get();
        counters::TIMEOUT_ROUNDS_COUNT.get();
        counters::TIMEOUT_COUNT.get();

        Self {
            time_interval,
            highest_committed_round: 0,
            current_round: 0,
            current_round_deadline: Instant::now(),
            time_service,
            timeout_sender,
        }
    }

    /// Return the current round.
    pub fn current_round(&self) -> Round {
        self.current_round
    }

    /// Returns deadline for current round
    pub fn current_round_deadline(&self) -> Instant {
        self.current_round_deadline
    }

    /// In case the local timeout corresponds to the current round, reset the timeout and
    /// return true. Otherwise ignore and return false.
    pub fn process_local_timeout(&mut self, round: Round) -> bool {
        if round != self.current_round {
            return false;
        }
        warn!("Local timeout for round {}", round);
        counters::TIMEOUT_COUNT.inc();
        self.setup_timeout();
        true
    }

    /// Notify the Pacemaker about the potentially new QC, TC, and highest committed round.
    /// Note that some of these values might not be available by the caller.
    pub fn process_certificates(
        &mut self,
        hqc_round: Option<Round>,
        htc_round: Option<Round>,
        highest_committed_round: Option<Round>,
    ) -> Option<NewRoundEvent> {
        let qc_round = hqc_round.unwrap_or(0);
        let tc_round = htc_round.unwrap_or(0);
        if let Some(committed_round) = highest_committed_round {
            if committed_round > self.highest_committed_round {
                self.highest_committed_round = committed_round;
            }
        }
        let new_round = std::cmp::max(qc_round, tc_round) + 1;
        if new_round > self.current_round {
            // Start a new round.
            self.current_round = new_round;
            let timeout = self.setup_timeout();
            // The new round reason is QCReady in case both QC and TC are equal
            let new_round_reason = if qc_round >= tc_round {
                NewRoundReason::QCReady
            } else {
                NewRoundReason::Timeout
            };
            let new_round_event = NewRoundEvent {
                round: self.current_round,
                reason: new_round_reason,
                timeout,
            };
            debug!("Starting new round: {}", new_round_event);
            return Some(new_round_event);
        }
        None
    }

    /// Setup the timeout task and return the duration of the current timeout
    fn setup_timeout(&mut self) -> Duration {
        let timeout_sender = self.timeout_sender.clone();
        let timeout = self.setup_deadline();
        trace!(
            "Scheduling timeout of {} ms for round {}",
            timeout.as_millis(),
            self.current_round
        );
        self.time_service
            .run_after(timeout, SendTask::make(timeout_sender, self.current_round));
        timeout
    }

    /// Setup the current round deadline and return the duration of the current round
    fn setup_deadline(&mut self) -> Duration {
        let round_index_after_committed_round = {
            if self.highest_committed_round == 0 {
                // Genesis doesn't require the 3-chain rule for commit, hence start the index at
                // the round after genesis.
                self.current_round - 1
            } else if self.current_round < self.highest_committed_round + 3 {
                0
            } else {
                self.current_round - self.highest_committed_round - 3
            }
        } as usize;
        let timeout = self
            .time_interval
            .get_round_duration(round_index_after_committed_round);
        let now = Instant::now();
        debug!(
            "{:?} passed since the previous deadline.",
            now.checked_duration_since(self.current_round_deadline)
                .map_or("0 ms".to_string(), |v| format!("{:?}", v))
        );
        debug!("Set round deadline to {:?} from now", timeout);
        self.current_round_deadline = now + timeout;
        timeout
    }
}
