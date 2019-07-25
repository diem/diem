// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::Round,
    liveness::timeout_msg::{PacemakerTimeout, PacemakerTimeoutCertificate},
};
use futures::Future;
use std::{
    fmt,
    pin::Pin,
    time::{Duration, Instant},
};

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

/// Pacemaker is responsible for generating the new round events, which are driving the actions
/// of the rest of the system (e.g., for generating new proposals).
/// Ideal pacemaker provides an abstraction of a "shared clock". In reality pacemaker
/// implementations use external signals like receiving new votes / QCs plus internal
/// communication between other nodes' pacemaker instances in order to synchronize the logical
/// clocks.
/// The trait doesn't specify the starting conditions or the executor that is responsible for
/// driving the logic.
pub trait Pacemaker: Send + Sync {
    /// Returns deadline for current round
    fn current_round_deadline(&self) -> Instant;

    /// Synchronous function to return the current round.
    fn current_round(&self) -> Round;

    /// Function to update current round based on received certificates.
    /// Both round of latest received QC and timeout certificates are taken into account.
    /// This function guarantees to update pacemaker state when promise that it returns is fulfilled
    fn process_certificates(
        &self,
        qc_round: Round,
        timeout_certificate: Option<&PacemakerTimeoutCertificate>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// The function is invoked upon receiving a remote timeout message from another validator.
    fn process_remote_timeout(
        &self,
        pacemaker_timeout: PacemakerTimeout,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    /// Update the highest committed round
    fn update_highest_committed_round(&self, highest_committed_round: Round);
}
