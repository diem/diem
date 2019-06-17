// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::Author,
    liveness::new_round_msg::{PacemakerTimeout, PacemakerTimeoutCertificate},
    persistent_storage::PersistentLivenessStorage,
};
use logger::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
#[path = "pacemaker_timeout_manager_test.rs"]
mod pacemaker_timeout_manager_test;

/// Tracks the highest round known local and received timeout certificates
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HighestTimeoutCertificates {
    // Highest timeout certificate gathered locally
    highest_local_timeout_certificate: Option<PacemakerTimeoutCertificate>,
    // Highest timeout certificate received from another replica
    highest_received_timeout_certificate: Option<PacemakerTimeoutCertificate>,
}

impl HighestTimeoutCertificates {
    #[cfg(test)]
    pub fn new(
        highest_local_timeout_certificate: Option<PacemakerTimeoutCertificate>,
        highest_received_timeout_certificate: Option<PacemakerTimeoutCertificate>,
    ) -> Self {
        Self {
            highest_local_timeout_certificate,
            highest_received_timeout_certificate,
        }
    }

    /// Return a optional reference to the highest timeout certificate (locally generated or
    /// remotely received)
    pub fn highest_timeout_certificate(&self) -> Option<&PacemakerTimeoutCertificate> {
        if let Some(highest_received_timeout_certificate) =
            self.highest_received_timeout_certificate.as_ref()
        {
            if let Some(highest_local_timeout_certificate) = &self.highest_local_timeout_certificate
            {
                if highest_local_timeout_certificate.round()
                    > highest_received_timeout_certificate.round()
                {
                    self.highest_local_timeout_certificate.as_ref()
                } else {
                    self.highest_received_timeout_certificate.as_ref()
                }
            } else {
                self.highest_received_timeout_certificate.as_ref()
            }
        } else {
            self.highest_local_timeout_certificate.as_ref()
        }
    }
}

/// Manages the PacemakerTimeout structs received from replicas.
///
/// A replica can generate and track TimeoutCertificates of the highest round (locally and received)
/// to allow a pacemaker to advance to the latest certificate round.
pub struct PacemakerTimeoutManager {
    // The minimum quorum to generate a timeout certificate
    timeout_certificate_quorum_size: usize,
    // Track the PacemakerTimeoutMsg for highest timeout round received from this node
    author_to_received_timeouts: HashMap<Author, PacemakerTimeout>,
    // Highest timeout certificates
    highest_timeout_certificates: HighestTimeoutCertificates,
    // Used to persistently store the latest known timeout certificate
    persistent_liveness_storage: Box<dyn PersistentLivenessStorage>,
}

impl PacemakerTimeoutManager {
    pub fn new(
        timeout_certificate_quorum_size: usize,
        highest_timeout_certificates: HighestTimeoutCertificates,
        persistent_liveness_storage: Box<dyn PersistentLivenessStorage>,
    ) -> Self {
        // This struct maintains the invariant that the highest round timeout certificate
        // that author_to_received_timeouts can generate is always equal to
        // highest_timeout_certificates.highest_local_timeout_certificate.
        let mut author_to_received_timeouts = HashMap::new();
        if let Some(tc) = &highest_timeout_certificates.highest_local_timeout_certificate {
            author_to_received_timeouts = tc
                .timeouts()
                .iter()
                .map(|t| (t.author(), t.clone()))
                .collect();
        }
        PacemakerTimeoutManager {
            timeout_certificate_quorum_size,
            author_to_received_timeouts,
            highest_timeout_certificates,
            persistent_liveness_storage,
        }
    }

    /// Returns the highest round PacemakerTimeoutCertificate from a map of author to
    /// timeout messages or None if there are not enough timeout messages available.
    /// A PacemakerTimeoutCertificate is made of the N highest timeout messages received where
    /// N=timeout_quorum_size.  The round of PacemakerTimeoutCertificate is determined as
    /// the smallest of round of all messages used to generate this certificate.
    ///
    /// For example, if timeout_certificate_quorum_size=3 and we received unique author timeouts
    /// for rounds (1,2,3,4), then rounds (2,3,4) would form PacemakerTimeoutCertificate with
    /// round=2.
    fn generate_timeout_certificate(
        author_to_received_timeouts: &HashMap<Author, PacemakerTimeout>,
        timeout_certificate_quorum_size: usize,
    ) -> Option<PacemakerTimeoutCertificate> {
        if author_to_received_timeouts.values().len() < timeout_certificate_quorum_size {
            return None;
        }
        let mut values: Vec<&PacemakerTimeout> = author_to_received_timeouts.values().collect();
        values.sort_by(|x, y| y.round().cmp(&x.round()));
        let slice = &values[..timeout_certificate_quorum_size];
        Some(PacemakerTimeoutCertificate::new(
            // expect does not panic here because code above verifies values length
            slice
                .last()
                .expect("Slice for timeout certificate is empty")
                .round(),
            slice.iter().map(|x| (*x).clone()).collect(),
        ))
    }

    /// Updates internal state according to received message from remote pacemaker and returns true
    /// if round derived from highest PacemakerTimeoutCertificate has increased.
    pub fn update_received_timeout(&mut self, pacemaker_timeout: PacemakerTimeout) -> bool {
        let author = pacemaker_timeout.author();
        let prev_timeout = self.author_to_received_timeouts.get(&author).cloned();
        if let Some(prev_timeout) = &prev_timeout {
            if prev_timeout.round() >= pacemaker_timeout.round() {
                warn!("Received timeout message for previous round, ignoring. Author: {}, prev round: {}, received: {}",
                          author.short_str(), prev_timeout.round(), pacemaker_timeout.round());
                return false;
            }
        }

        self.author_to_received_timeouts
            .insert(author, pacemaker_timeout.clone());
        let highest_timeout_certificate = Self::generate_timeout_certificate(
            &self.author_to_received_timeouts,
            self.timeout_certificate_quorum_size,
        );
        let highest_round = match &highest_timeout_certificate {
            Some(tc) => tc.round(),
            None => return false,
        };
        let prev_highest_round = self
            .highest_timeout_certificates
            .highest_local_timeout_certificate
            .as_ref()
            .map(PacemakerTimeoutCertificate::round);
        assert!(
            highest_round >= prev_highest_round.unwrap_or(0),
            "Went down on highest timeout quorum round from {:?} to {:?}.
            Received: {:?}, all: {:?}",
            prev_highest_round,
            highest_round,
            pacemaker_timeout,
            self.author_to_received_timeouts,
        );
        self.highest_timeout_certificates
            .highest_local_timeout_certificate = highest_timeout_certificate;
        if let Err(e) = self
            .persistent_liveness_storage
            .save_highest_timeout_cert(self.highest_timeout_certificates.clone())
        {
            warn!(
                "Failed to persist local highest timeout certificate in round {} due to {}",
                highest_round, e
            );
        }
        highest_round > prev_highest_round.unwrap_or(0)
    }

    /// Attempts to update highest_received_timeout_certificate when receiving a new remote
    /// timeout certificate.  Returns true if highest_received_timeout_certificate has changed
    pub fn update_highest_received_timeout_certificate(
        &mut self,
        timeout_certificate: &PacemakerTimeoutCertificate,
    ) -> bool {
        if timeout_certificate.round()
            > self
                .highest_timeout_certificates
                .highest_received_timeout_certificate
                .as_ref()
                .map_or(0, PacemakerTimeoutCertificate::round)
        {
            debug!(
                "Received remote timeout certificate at round {}",
                timeout_certificate.round()
            );
            self.highest_timeout_certificates
                .highest_received_timeout_certificate = Some(timeout_certificate.clone());
            if let Err(e) = self
                .persistent_liveness_storage
                .save_highest_timeout_cert(self.highest_timeout_certificates.clone())
            {
                warn!(
                    "Failed to persist received highest timeout certificate in round {} due to {}",
                    timeout_certificate.round(),
                    e
                );
            }
            return true;
        }
        false
    }

    /// Return a optional reference to the highest timeout certificate (locally generated or
    /// remotely received)
    pub fn highest_timeout_certificate(&self) -> Option<&PacemakerTimeoutCertificate> {
        self.highest_timeout_certificates
            .highest_timeout_certificate()
    }
}
