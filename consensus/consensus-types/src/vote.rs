// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::quorum_cert::QuorumCert;
use crate::{common::Author, timeout::Timeout, vote_data::VoteData};
use anyhow::{ensure, Context};
use diem_crypto::{ed25519::Ed25519Signature, hash::CryptoHash};
use diem_types::{
    ledger_info::LedgerInfo, validator_signer::ValidatorSigner,
    validator_verifier::ValidatorVerifier,
};
use serde::{Deserialize, Serialize};
use short_hex_str::AsShortHexStr;
use std::fmt::{Debug, Display, Formatter};

/// Vote is the struct that is ultimately sent by the voter in response for
/// receiving a proposal.
/// Vote carries the `LedgerInfo` of a block that is going to be committed in case this vote
/// is gathers QuorumCertificate (see the detailed explanation in the comments of `LedgerInfo`).
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Vote {
    /// The data of the vote
    vote_data: VoteData,
    /// The identity of the voter.
    author: Author,
    /// LedgerInfo of a block that is going to be committed in case this vote gathers QC.
    ledger_info: LedgerInfo,
    /// Signature of the LedgerInfo
    signature: Ed25519Signature,
    /// The signature signed on Timeout can be aggregated into a timeout certificate if present.
    timeout: Option<(Timeout, Ed25519Signature)>,
}

// this is required by structured log
impl Debug for Vote {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Vote {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Vote: [vote data: {}, author: {}, is_timeout: {}, {}]",
            self.vote_data,
            self.author.short_str(),
            self.is_timeout(),
            self.ledger_info
        )
    }
}

impl Vote {
    /// Generates a new Vote corresponding to the "fast-vote" path without the round signatures
    /// that can be aggregated into a timeout certificate
    pub fn new(
        vote_data: VoteData,
        author: Author,
        mut ledger_info_placeholder: LedgerInfo,
        validator_signer: &ValidatorSigner,
    ) -> Self {
        ledger_info_placeholder.set_consensus_data_hash(vote_data.hash());
        let signature = validator_signer.sign(&ledger_info_placeholder);
        Self::new_with_signature(vote_data, author, ledger_info_placeholder, signature)
    }

    /// Generates a new Vote using a signature over the specified ledger_info
    pub fn new_with_signature(
        vote_data: VoteData,
        author: Author,
        ledger_info: LedgerInfo,
        signature: Ed25519Signature,
    ) -> Self {
        Self {
            vote_data,
            author,
            ledger_info,
            signature,
            timeout: None,
        }
    }

    /// Generates a round signature, which can then be used for aggregating a timeout certificate.
    /// Typically called for generating vote messages that are sent upon timeouts.
    pub fn add_timeout(&mut self, timeout: Timeout, signature: Ed25519Signature) {
        // if self.timeout.is_some() {
        //     return; // round signature is already set
        // }
        assert_eq!(self.vote_data.proposed().epoch(), timeout.epoch());
        assert_eq!(self.vote_data.proposed().round(), timeout.round());

        self.timeout.replace((timeout, signature));
    }

    pub fn vote_data(&self) -> &VoteData {
        &self.vote_data
    }

    /// Return the author of the vote
    pub fn author(&self) -> Author {
        self.author
    }

    /// Return the LedgerInfo associated with this vote
    pub fn ledger_info(&self) -> &LedgerInfo {
        &self.ledger_info
    }

    /// Return the signature of the vote
    pub fn signature(&self) -> &Ed25519Signature {
        &self.signature
    }

    /// Generate Timeout with the highest quorum cert.
    pub fn generate_timeout(&self, quorum_cert: &QuorumCert) -> Timeout {
        Timeout::new(
            self.epoch(),
            self.vote_data().proposed().round(),
            quorum_cert.certified_block().round(),
        )
    }

    /// Returns the Timeout and corresponding signatures
    pub fn timeout_and_signature(&self) -> Option<(Timeout, Ed25519Signature)> {
        self.timeout.clone()
    }

    pub fn timeout(&self) -> Option<&Timeout> {
        self.timeout.as_ref().map(|(t, _)| t)
    }

    /// Return the epoch of the vote
    pub fn epoch(&self) -> u64 {
        self.vote_data.proposed().epoch()
    }

    /// Returns the signature for the vote_data().proposed().round() that can be aggregated for
    /// TimeoutCertificate.
    pub fn timeout_signature(&self) -> Option<&Ed25519Signature> {
        self.timeout.as_ref().map(|(_, s)| s)
    }

    /// The vote message is considered a timeout vote message if it carries a signature on the
    /// round, which can then be used for aggregating it to the TimeoutCertificate.
    pub fn is_timeout(&self) -> bool {
        self.timeout.is_some()
    }

    /// Verifies that the consensus data hash of LedgerInfo corresponds to the vote info,
    /// and then verifies the signature.
    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        ensure!(
            self.ledger_info.consensus_data_hash() == self.vote_data.hash(),
            "Vote's hash mismatch with LedgerInfo"
        );
        validator
            .verify(self.author(), &self.ledger_info, &self.signature)
            .context("Failed to verify Vote")?;
        if let Some((timeout, timeout_signature)) = &self.timeout {
            ensure!(
                timeout.epoch() == self.epoch(),
                "Timeout in vote has different epoch"
            );
            ensure!(
                timeout.round() == self.vote_data.proposed().round(),
                "Timeout in vote has different round"
            );
            // verify the message
            validator
                .verify(self.author(), timeout, timeout_signature)
                .context("Failed to verify Timeout signature")?;
        }
        // Let us verify the vote data as well
        self.vote_data().verify()?;
        Ok(())
    }
}
