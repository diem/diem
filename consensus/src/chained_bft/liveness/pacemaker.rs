// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::Round,
        consensus_types::timeout_msg::{PacemakerTimeout, PacemakerTimeoutCertificate},
        liveness::pacemaker_timeout_manager::{
            HighestTimeoutCertificates, PacemakerTimeoutManager,
        },
        persistent_storage::PersistentLivenessStorage,
    },
    counters,
    util::time_service::{SendTask, TimeService},
};
use channel;
use libra_types::crypto_proxies::ValidatorVerifier;
use logger::prelude::*;
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};
use termion::color::*;

/// A reason for starting a new round: introduced for monitoring / debug purposes.
#[derive(Eq, Debug, PartialEq)]
pub enum NewRoundReason {
    QCReady,
    Timeout { cert: PacemakerTimeoutCertificate },
}

impl fmt::Display for NewRoundReason {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NewRoundReason::QCReady => write!(f, "QCReady"),
            NewRoundReason::Timeout { cert } => write!(f, "{}", cert),
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

/// `Pacemaker` is a Pacemaker implementation that relies on increasing local timeouts
/// in order to eventually come up with the timeout that is large enough to guarantee overlap of the
/// "current round" of multiple participants.
///
/// The protocol is as follows:
/// * `Pacemaker` manages the `highest_certified_round` that is keeping the round of the
/// highest certified block known to the validator.
/// * Once a new QC arrives with a round larger than that of `highest_certified_round`,
/// local pacemaker is going to increment a round with a default timeout.
/// * Upon every timeout `Pacemaker` increments a round and doubles the timeout.
///
/// `Pacemaker` does not require clock synchronization to maintain the property of
/// liveness - although clock synchronization can improve the time necessary to get a large enough
/// timeout overlap.
/// It does rely on an assumption that when an honest replica receives a quorum certificate
/// indicating to move to the next round, all other honest replicas will move to the next round
/// within a bounded time. This can be guaranteed via all honest replicas gossiping their highest
/// QC to f+1 other replicas for instance.
pub struct Pacemaker {
    // Determines the time interval for a round interval
    time_interval: Box<dyn PacemakerTimeInterval>,
    // Highest round that a block was committed
    highest_committed_round: Round,
    // Highest round known certified by QC.
    highest_qc_round: Round,
    // Current round (current_round - highest_qc_round determines the timeout).
    // Current round is basically max(highest_qc_round, highest_received_tc, highest_local_tc) + 1
    // update_current_round take care of updating current_round and sending new round event if
    // it changes
    current_round: Round,
    // Approximate deadline when current round ends
    current_round_deadline: Instant,
    // Service for timer
    time_service: Arc<dyn TimeService>,
    // To send timeout events to other pacemakers
    timeout_sender: channel::Sender<Round>,
    // Manages the PacemakerTimeout and PacemakerTimeoutCertificate structs
    pacemaker_timeout_manager: PacemakerTimeoutManager,
}

impl Pacemaker {
    pub fn new(
        persistent_liveness_storage: Box<dyn PersistentLivenessStorage>,
        time_interval: Box<dyn PacemakerTimeInterval>,
        time_service: Arc<dyn TimeService>,
        timeout_sender: channel::Sender<Round>,
        highest_timeout_certificate: HighestTimeoutCertificates,
    ) -> Self {
        // Our counters are initialized via lazy_static, so they're not going to appear in
        // Prometheus if some conditions never happen. Invoking get() function enforces creation.
        counters::QC_ROUNDS_COUNT.get();
        counters::TIMEOUT_ROUNDS_COUNT.get();
        counters::TIMEOUT_COUNT.get();

        Self {
            time_interval,
            highest_committed_round: 0,
            highest_qc_round: 0,
            current_round: 0,
            current_round_deadline: Instant::now(),
            time_service,
            timeout_sender,
            pacemaker_timeout_manager: PacemakerTimeoutManager::new(
                highest_timeout_certificate,
                persistent_liveness_storage,
            ),
        }
    }

    /// Setup the timeout task and return the duration of the current timeout
    fn setup_timeout(&mut self) -> Duration {
        let timeout_sender = self.timeout_sender.clone();
        let timeout = self.setup_deadline();
        // Note that the timeout should not be driven sequentially with any other events as it can
        // become the head of the line blocker.
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
        self.current_round_deadline = Instant::now() + timeout;
        timeout
    }

    /// Attempts to update highest_qc_certified_round when receiving QC for given round.
    /// Returns true if highest_qc_certified_round of this pacemaker has changed
    fn update_highest_qc_round(&mut self, round: Round) {
        if round > self.highest_qc_round {
            debug!(
                "{}QuorumCertified at {}{}",
                Fg(LightBlack),
                round,
                Fg(Reset)
            );
            self.highest_qc_round = round;
        }
    }

    /// Combines highest_qc_certified_round, highest_local_tc and highest_received_tc into
    /// effective round of this pacemaker.
    /// Generates new_round event if effective round changes and ensures it is
    /// monotonically increasing
    fn update_current_round(&mut self) -> Option<NewRoundEvent> {
        let (mut best_round, mut best_reason) = (self.highest_qc_round, NewRoundReason::QCReady);
        if let Some(highest_timeout_certificate) =
            self.pacemaker_timeout_manager.highest_timeout_certificate()
        {
            if highest_timeout_certificate.round() > best_round {
                best_round = highest_timeout_certificate.round();
                best_reason = NewRoundReason::Timeout {
                    cert: highest_timeout_certificate.clone(),
                };
            }
        }

        let new_round = best_round + 1;
        if self.current_round == new_round {
            return None;
        }
        assert!(
            new_round > self.current_round,
            "Round illegally decreased from {} to {}",
            self.current_round,
            new_round
        );
        self.current_round = new_round;
        let timeout = self.setup_timeout();
        Some(NewRoundEvent {
            round: self.current_round,
            reason: best_reason,
            timeout,
        })
    }

    /// Validate timeout certificate and update local state if it's correct
    fn check_and_update_highest_received_tc(&mut self, tc: Option<&PacemakerTimeoutCertificate>) {
        if let Some(tc) = tc {
            self.pacemaker_timeout_manager
                .update_highest_received_timeout_certificate(tc);
        }
    }

    /// Returns deadline for current round
    pub fn current_round_deadline(&self) -> Instant {
        self.current_round_deadline
    }

    /// Synchronous function to return the current round.
    pub fn current_round(&self) -> Round {
        self.current_round
    }

    /// Return a optional reference to the highest timeout certificate (locally generated or
    /// remotely received)
    pub fn highest_timeout_certificate(&self) -> Option<PacemakerTimeoutCertificate> {
        self.pacemaker_timeout_manager
            .highest_timeout_certificate()
            .cloned()
    }

    /// Function to update current round based on received certificates.
    /// Both round of latest received QC and timeout certificates are taken into account.
    /// This function guarantees to update pacemaker state when promise that it returns is fulfilled
    pub fn process_certificates(
        &mut self,
        qc_round: Round,
        highest_committed_round: Option<Round>,
        timeout_certificate: Option<&PacemakerTimeoutCertificate>,
    ) -> Option<NewRoundEvent> {
        self.check_and_update_highest_received_tc(timeout_certificate);
        self.update_highest_qc_round(qc_round);
        match highest_committed_round {
            Some(commit_round) if (commit_round > self.highest_committed_round) => {
                self.highest_committed_round = commit_round;
            }
            _ => (),
        }
        self.update_current_round()
    }

    /// The function is invoked upon receiving a remote timeout message from another validator.
    pub fn process_remote_timeout(
        &mut self,
        pacemaker_timeout: PacemakerTimeout,
        validator_verifier: Arc<ValidatorVerifier>,
    ) -> Option<NewRoundEvent> {
        if self
            .pacemaker_timeout_manager
            .update_received_timeout(pacemaker_timeout, validator_verifier)
        {
            self.update_current_round()
        } else {
            None
        }
    }

    /// To process the local round timeout triggered by TimeService and return whether it's the
    /// current round.
    pub fn process_local_timeout(&mut self, round: Round) -> bool {
        if round != self.current_round {
            return false;
        }
        warn!(
            "Round {} has timed out, broadcasting new round message to all replicas",
            round
        );
        counters::TIMEOUT_COUNT.inc();
        self.setup_timeout();
        true
    }
}
