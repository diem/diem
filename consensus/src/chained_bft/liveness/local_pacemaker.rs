// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        common::Round,
        liveness::{
            pacemaker::{NewRoundEvent, NewRoundReason, Pacemaker},
            pacemaker_timeout_manager::{HighestTimeoutCertificates, PacemakerTimeoutManager},
            timeout_msg::{PacemakerTimeout, PacemakerTimeoutCertificate},
        },
        persistent_storage::PersistentLivenessStorage,
    },
    counters,
    time_service::{SendTask, TimeService},
};
use channel;
use futures::{Future, FutureExt, SinkExt, StreamExt, TryFutureExt};
use logger::prelude::*;
use std::{
    cmp::{self, max},
    pin::Pin,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use termion::color::*;
use tokio::runtime::TaskExecutor;

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
    #[cfg(test)]
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

/// `LocalPacemakerInner` is a Pacemaker implementation that relies on increasing local timeouts
/// in order to eventually come up with the timeout that is large enough to guarantee overlap of the
/// "current round" of multiple participants.
///
/// The protocol is as follows:
/// * `LocalPacemakerInner` manages the `highest_certified_round` that is keeping the round of the
/// highest certified block known to the validator.
/// * Once a new QC arrives with a round larger than that of `highest_certified_round`,
/// local pacemaker is going to increment a round with a default timeout.
/// * Upon every timeout `LocalPacemaker` increments a round and doubles the timeout.
///
/// `LocalPacemakerInner` does not require clock synchronization to maintain the property of
/// liveness - although clock synchronization can improve the time necessary to get a large enough
/// timeout overlap.
/// It does rely on an assumption that when an honest replica receives a quorum certificate
/// indicating to move to the next round, all other honest replicas will move to the next round
/// within a bounded time. This can be guaranteed via all honest replicas gossiping their highest
/// QC to f+1 other replicas for instance.
struct LocalPacemakerInner {
    // Determines the time interval for a round interval
    time_interval: Box<PacemakerTimeInterval>,
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
    // To send new round events.
    new_round_events_sender: channel::Sender<NewRoundEvent>,
    // To send timeout events to itself.
    local_timeout_sender: channel::Sender<Round>,
    // To send timeout events to other pacemakers
    external_timeout_sender: channel::Sender<Round>,
    // Manages the PacemakerTimeout and PacemakerTimeoutCertificate structs
    pacemaker_timeout_manager: PacemakerTimeoutManager,
}

impl LocalPacemakerInner {
    pub fn new(
        persistent_liveness_storage: Box<dyn PersistentLivenessStorage>,
        time_interval: Box<PacemakerTimeInterval>,
        highest_committed_round: Round,
        highest_qc_round: Round,
        time_service: Arc<dyn TimeService>,
        new_round_events_sender: channel::Sender<NewRoundEvent>,
        local_timeout_sender: channel::Sender<Round>,
        external_timeout_sender: channel::Sender<Round>,
        pacemaker_timeout_quorum_size: usize,
        highest_timeout_certificates: HighestTimeoutCertificates,
    ) -> Self {
        assert!(pacemaker_timeout_quorum_size > 0);
        // The starting round is maximum(highest quorum certificate,
        // highest timeout certificate round) + 1.  Note that it is possible this
        // replica already voted at this round and will until a round timeout
        // or another replica convinces it via a quorum certificate or a timeout
        // certificate to advance to a higher round.
        let current_round = {
            match highest_timeout_certificates.highest_timeout_certificate() {
                Some(highest_timeout_certificate) => {
                    cmp::max(highest_qc_round, highest_timeout_certificate.round())
                }
                None => highest_qc_round,
            }
        } + 1;
        // Our counters are initialized via lazy_static, so they're not going to appear in
        // Prometheus if some conditions never happen. Invoking get() function enforces creation.
        counters::QC_ROUNDS_COUNT.get();
        counters::TIMEOUT_ROUNDS_COUNT.get();
        counters::TIMEOUT_COUNT.get();

        Self {
            time_interval,
            highest_committed_round,
            highest_qc_round,
            current_round,
            current_round_deadline: Instant::now(),
            time_service,
            new_round_events_sender,
            local_timeout_sender,
            external_timeout_sender,
            pacemaker_timeout_manager: PacemakerTimeoutManager::new(
                pacemaker_timeout_quorum_size,
                highest_timeout_certificates,
                persistent_liveness_storage,
            ),
        }
    }

    /// Trigger an event to create a new round interval and ignore any events from previous round
    /// intervals.  The reason for the event is given by the caller, the timeout is
    /// deterministically determined by the reason and the internal state.
    fn create_new_round_task(&mut self, reason: NewRoundReason) -> impl Future<Output = ()> + Send {
        let round = self.current_round;
        let timeout = self.setup_timeout();
        let mut sender = self.new_round_events_sender.clone();
        async move {
            if let Err(e) = sender
                .send(NewRoundEvent {
                    round,
                    reason,
                    timeout,
                })
                .await
            {
                debug!("Error in sending new round interval event: {:?}", e);
            }
        }
    }

    /// Process local timeout for a given round: in case a new timeout message should be sent
    /// return a channel for sending the timeout message.
    fn process_local_timeout(&mut self, round: Round) -> Option<channel::Sender<Round>> {
        if round != self.current_round {
            return None;
        }
        warn!(
            "Round {} has timed out, broadcasting new round message to all replicas",
            round
        );
        counters::TIMEOUT_COUNT.inc();
        self.setup_timeout();
        Some(self.external_timeout_sender.clone())
    }

    /// Setup the timeout task and return the duration of the current timeout
    fn setup_timeout(&mut self) -> Duration {
        let timeout_sender = self.local_timeout_sender.clone();
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
            } else {
                if self.current_round - self.highest_committed_round < 3 {
                    warn!("Finding a deadline for a round {} that should have already been completed since the highest committed round is {}",
                          self.current_round,
                          self.highest_committed_round);
                }

                max(0, self.current_round - self.highest_committed_round - 3)
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
    fn update_highest_qc_round(&mut self, round: Round) -> bool {
        if round > self.highest_qc_round {
            debug!(
                "{}QuorumCertified at {}{}",
                Fg(LightBlack),
                round,
                Fg(Reset)
            );
            self.highest_qc_round = round;
            return true;
        }
        false
    }

    /// Combines highest_qc_certified_round, highest_local_tc and highest_received_tc into
    /// effective round of this pacemaker.
    /// Generates new_round event if effective round changes and ensures it is
    /// monotonically increasing
    fn update_current_round(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
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
            debug!(
                "{}Round did not change: {}{}",
                Fg(LightBlack),
                new_round,
                Fg(Reset)
            );
            return async {}.boxed();
        }
        assert!(
            new_round > self.current_round,
            "Round illegally decreased from {} to {}",
            self.current_round,
            new_round
        );
        self.current_round = new_round;
        self.create_new_round_task(best_reason).boxed()
    }

    /// Validate timeout certificate and update local state if it's correct
    fn check_and_update_highest_received_tc(
        &mut self,
        tc: Option<&PacemakerTimeoutCertificate>,
    ) -> bool {
        if let Some(tc) = tc {
            return self
                .pacemaker_timeout_manager
                .update_highest_received_timeout_certificate(tc);
        }
        false
    }
}

/// `LocalPacemaker` is a wrapper to make the `LocalPacemakerInner` thread-safe.
pub struct LocalPacemaker {
    inner: Arc<RwLock<LocalPacemakerInner>>,
}

impl LocalPacemaker {
    pub fn new(
        executor: TaskExecutor,
        persistent_liveness_storage: Box<dyn PersistentLivenessStorage>,
        time_interval: Box<PacemakerTimeInterval>,
        highest_committed_round: Round,
        highest_qc_round: Round,
        time_service: Arc<dyn TimeService>,
        new_round_events_sender: channel::Sender<NewRoundEvent>,
        external_timeout_sender: channel::Sender<Round>,
        pacemaker_timeout_quorum_size: usize,
        highest_timeout_certificates: HighestTimeoutCertificates,
    ) -> Self {
        let (local_timeouts_sender, mut local_timeouts_receiver) =
            channel::new(1_024, &counters::PENDING_PACEMAKER_TIMEOUTS);
        let inner = Arc::new(RwLock::new(LocalPacemakerInner::new(
            persistent_liveness_storage,
            time_interval,
            highest_committed_round,
            highest_qc_round,
            time_service,
            new_round_events_sender,
            local_timeouts_sender,
            external_timeout_sender,
            pacemaker_timeout_quorum_size,
            highest_timeout_certificates,
        )));

        let inner_ref = Arc::clone(&inner);
        let timeout_processing_loop = async move {
            // To jump start the execution return the new round event for the current round.
            inner_ref
                .write()
                .unwrap()
                .create_new_round_task(NewRoundReason::QCReady)
                .await;

            // Start the loop of processing local timeouts
            while let Some(round) = local_timeouts_receiver.next().await {
                Self::process_local_timeout(Arc::clone(&inner_ref), round).await;
            }
        };
        executor.spawn(timeout_processing_loop.boxed().unit_error().compat());

        Self { inner }
    }

    async fn process_local_timeout(inner: Arc<RwLock<LocalPacemakerInner>>, round: Round) {
        let timeout_processing_res = { inner.write().unwrap().process_local_timeout(round) };
        if let Some(mut sender) = timeout_processing_res {
            if let Err(e) = sender.send(round).await {
                warn!("Can't send pacemaker timeout message: {:?}", e)
            }
        }
    }
}

impl Pacemaker for LocalPacemaker {
    fn current_round_deadline(&self) -> Instant {
        self.inner.read().unwrap().current_round_deadline
    }

    fn current_round(&self) -> Round {
        self.inner.read().unwrap().current_round
    }

    fn process_certificates(
        &self,
        qc_round: Round,
        timeout_certificate: Option<&PacemakerTimeoutCertificate>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let mut guard = self.inner.write().unwrap();
        let tc_round_updated = guard.check_and_update_highest_received_tc(timeout_certificate);
        let qc_round_updated = guard.update_highest_qc_round(qc_round);
        if tc_round_updated || qc_round_updated {
            return guard.update_current_round();
        }
        async {}.boxed()
    }

    /// The function is invoked upon receiving a remote timeout message from another validator.
    fn process_remote_timeout(
        &self,
        pacemaker_timeout: PacemakerTimeout,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let mut guard = self.inner.write().unwrap();
        if guard
            .pacemaker_timeout_manager
            .update_received_timeout(pacemaker_timeout)
        {
            return guard.update_current_round();
        }
        async {}.boxed()
    }

    fn update_highest_committed_round(&self, highest_committed_round: Round) {
        let mut guard = self.inner.write().unwrap();
        if guard.highest_committed_round < highest_committed_round {
            guard.highest_committed_round = highest_committed_round;
        }
    }
}
