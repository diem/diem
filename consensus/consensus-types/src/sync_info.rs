// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{common::Round, quorum_cert::QuorumCert, timeout_certificate::TimeoutCertificate};
use anyhow::{ensure, Context};
use libra_types::{block_info::BlockInfo, validator_verifier::ValidatorVerifier};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq)]
/// This struct describes basic synchronization metadata.
pub struct SyncInfo {
    /// Highest quorum certificate known to the peer.
    highest_quorum_cert: QuorumCert,
    /// Highest ledger info known to the peer.
    highest_commit_cert: Option<QuorumCert>,
    /// Optional highest timeout certificate if available.
    highest_timeout_cert: Option<TimeoutCertificate>,
}

// this is required by structured log
impl Debug for SyncInfo {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for SyncInfo {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let htc_repr = match self.highest_timeout_certificate() {
            Some(tc) => format!("{}", tc.round()),
            None => "None".to_string(),
        };
        write!(
            f,
            "SyncInfo[HQC: {}, HCC: {}, HTC: {}]",
            self.highest_certified_round(),
            self.highest_commit_round(),
            htc_repr,
        )
    }
}

impl SyncInfo {
    pub fn new(
        highest_quorum_cert: QuorumCert,
        highest_commit_cert: QuorumCert,
        highest_timeout_cert: Option<TimeoutCertificate>,
    ) -> Self {
        let commit_cert = if highest_quorum_cert == highest_commit_cert {
            None
        } else {
            Some(highest_commit_cert)
        };
        // No need to include HTC if it's lower than HQC
        let highest_timeout_cert = highest_timeout_cert
            .filter(|tc| tc.round() > highest_quorum_cert.certified_block().round());
        Self {
            highest_quorum_cert,
            highest_commit_cert: commit_cert,
            highest_timeout_cert,
        }
    }

    /// Highest quorum certificate
    pub fn highest_quorum_cert(&self) -> &QuorumCert {
        &self.highest_quorum_cert
    }

    /// Highest ledger info
    pub fn highest_commit_cert(&self) -> &QuorumCert {
        self.highest_commit_cert
            .as_ref()
            .unwrap_or(&self.highest_quorum_cert)
    }

    /// Highest timeout certificate if available
    pub fn highest_timeout_certificate(&self) -> Option<&TimeoutCertificate> {
        self.highest_timeout_cert.as_ref()
    }

    pub fn highest_certified_round(&self) -> Round {
        self.highest_quorum_cert.certified_block().round()
    }

    pub fn highest_timeout_round(&self) -> Round {
        self.highest_timeout_certificate()
            .map_or(0, |tc| tc.round())
    }

    pub fn highest_commit_round(&self) -> Round {
        self.highest_commit_cert().commit_info().round()
    }

    /// The highest round the SyncInfo carries.
    pub fn highest_round(&self) -> Round {
        std::cmp::max(self.highest_certified_round(), self.highest_timeout_round())
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        let epoch = self.highest_quorum_cert.certified_block().epoch();
        ensure!(
            epoch == self.highest_commit_cert().certified_block().epoch(),
            "Multi epoch in SyncInfo - HCC and HQC"
        );
        if let Some(tc) = &self.highest_timeout_cert {
            ensure!(epoch == tc.epoch(), "Multi epoch in SyncInfo - TC and HQC");
        }

        ensure!(
            self.highest_quorum_cert.certified_block().round()
                >= self.highest_commit_cert().certified_block().round(),
            "HQC has lower round than HCC"
        );
        ensure!(
            *self.highest_commit_cert().commit_info() != BlockInfo::empty(),
            "HCC has no committed block"
        );
        self.highest_quorum_cert
            .verify(validator)
            .and_then(|_| {
                self.highest_commit_cert
                    .as_ref()
                    .map_or(Ok(()), |cert| cert.verify(validator))
            })
            .and_then(|_| {
                if let Some(tc) = &self.highest_timeout_cert {
                    tc.verify(validator)?;
                }
                Ok(())
            })
            .context("Fail to verify SyncInfo")?;
        Ok(())
    }

    pub fn epoch(&self) -> u64 {
        self.highest_quorum_cert.certified_block().epoch()
    }

    pub fn has_newer_certificates(&self, other: &SyncInfo) -> bool {
        self.highest_certified_round() > other.highest_certified_round()
            || self.highest_timeout_round() > other.highest_timeout_round()
            || self.highest_commit_round() > other.highest_commit_round()
    }
}
