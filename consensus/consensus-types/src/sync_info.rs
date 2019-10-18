// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{common::Round, quorum_cert::QuorumCert, timeout_certificate::TimeoutCertificate};
use failure::ResultExt;
use libra_types::crypto_proxies::ValidatorVerifier;
use network;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
};

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
/// This struct describes basic synchronization metadata.
pub struct SyncInfo {
    /// Highest quorum certificate known to the peer.
    highest_quorum_cert: QuorumCert,
    /// Highest ledger info known to the peer.
    highest_ledger_info: QuorumCert,
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
            "SyncInfo[round: {}, HQC: {}, HLI: {}, HTC: {}]",
            self.highest_round(),
            self.highest_quorum_cert,
            self.highest_ledger_info,
            htc_repr,
        )
    }
}

impl SyncInfo {
    pub fn new(
        highest_quorum_cert: QuorumCert,
        highest_ledger_info: QuorumCert,
        highest_timeout_cert: Option<TimeoutCertificate>,
    ) -> Self {
        Self {
            highest_quorum_cert,
            highest_ledger_info,
            highest_timeout_cert,
        }
    }

    /// Highest quorum certificate
    pub fn highest_quorum_cert(&self) -> &QuorumCert {
        &self.highest_quorum_cert
    }

    /// Highest ledger info
    pub fn highest_ledger_info(&self) -> &QuorumCert {
        &self.highest_ledger_info
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

    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        self.highest_quorum_cert
            .verify(validator)
            .and_then(|_| self.highest_ledger_info.verify(validator))
            .and_then(|_| {
                if let Some(tc) = &self.highest_timeout_cert {
                    tc.verify(validator)?;
                }
                Ok(())
            })
            .with_context(|e| format!("Fail to verify SyncInfo: {:?}", e))?;
        Ok(())
    }
}

impl TryFrom<network::proto::SyncInfo> for SyncInfo {
    type Error = failure::Error;

    fn try_from(proto: network::proto::SyncInfo) -> failure::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl TryFrom<SyncInfo> for network::proto::SyncInfo {
    type Error = failure::Error;

    fn try_from(info: SyncInfo) -> failure::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&info)?,
        })
    }
}
