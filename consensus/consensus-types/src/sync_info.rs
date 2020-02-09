// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{common::Round, quorum_cert::QuorumCert, timeout_certificate::TimeoutCertificate};
use anyhow::{ensure, Context};
use libra_types::block_info::BlockInfo;
use libra_types::crypto_proxies::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
/// This struct describes basic synchronization metadata.
pub struct SyncInfo {
    /// Highest quorum certificate known to the peer.
    highest_quorum_cert: QuorumCert,
    /// Highest ledger info known to the peer.
    highest_commit_cert: QuorumCert,
    /// Optional highest timeout certificate if available.
    highest_timeout_cert: Option<TimeoutCertificate>,
}

impl Display for SyncInfo {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let htc_repr = match self.highest_timeout_certificate() {
            Some(tc) => format!("TC for round {}", tc.round()),
            None => "None".to_string(),
        };
        write!(
            f,
            "SyncInfo[round: {}, HQC: {}, HCC: {}, HTC: {}]",
            self.highest_round(),
            self.highest_quorum_cert,
            self.highest_commit_cert,
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
        Self {
            highest_quorum_cert,
            highest_commit_cert,
            highest_timeout_cert,
        }
    }

    /// Highest quorum certificate
    pub fn highest_quorum_cert(&self) -> &QuorumCert {
        &self.highest_quorum_cert
    }

    /// Highest ledger info
    pub fn highest_commit_cert(&self) -> &QuorumCert {
        &self.highest_commit_cert
    }

    /// Highest timeout certificate if available
    pub fn highest_timeout_certificate(&self) -> Option<&TimeoutCertificate> {
        self.highest_timeout_cert.as_ref()
    }

    pub fn hqc_round(&self) -> Round {
        self.highest_quorum_cert.certified_block().round()
    }

    pub fn htc_round(&self) -> Round {
        self.highest_timeout_certificate()
            .map_or(0, |tc| tc.round())
    }

    /// The highest round the SyncInfo carries.
    pub fn highest_round(&self) -> Round {
        std::cmp::max(self.hqc_round(), self.htc_round())
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        let epoch = self.highest_quorum_cert.certified_block().epoch();
        ensure!(
            epoch == self.highest_commit_cert.certified_block().epoch(),
            "Multi epoch in SyncInfo - HCC and HQC"
        );
        if let Some(tc) = &self.highest_timeout_cert {
            ensure!(epoch == tc.epoch(), "Multi epoch in SyncInfo - TC and HQC");
        }

        ensure!(
            self.highest_quorum_cert.certified_block().round()
                >= self.highest_commit_cert.certified_block().round(),
            "HQC has lower round than HCC"
        );
        ensure!(
            *self.highest_commit_cert.commit_info() != BlockInfo::empty(),
            "HCC has no committed block"
        );
        self.highest_quorum_cert
            .verify(validator)
            .and_then(|_| self.highest_commit_cert.verify(validator))
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
}
