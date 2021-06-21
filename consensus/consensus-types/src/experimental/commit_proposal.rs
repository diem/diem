// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::{Author, Round};
use anyhow::Context;
use diem_crypto::ed25519::Ed25519Signature;
use std::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Serialize};
use short_hex_str::AsShortHexStr;
use diem_types::{
    ledger_info::LedgerInfo, validator_signer::ValidatorSigner,
    validator_verifier::ValidatorVerifier,
};


#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CommitProposal {
    author: Author,
    ledger_info: LedgerInfo,
    signature: Ed25519Signature,
}

// this is required by structured log
impl Debug for CommitProposal {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for CommitProposal {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "CommitProposal: [author: {}, {}]",
            self.author.short_str(),
            self.ledger_info
        )
    }
}

impl CommitProposal {
    /// Generates a new CommitProposal
    pub fn new(
        author: Author,
        ledger_info_placeholder: LedgerInfo,
        validator_signer: &ValidatorSigner,
    ) -> Self {
        let signature = validator_signer.sign(&ledger_info_placeholder);
        Self::new_with_signature(author, ledger_info_placeholder, signature)
    }

    /// Generates a new CommitProposal using a signature over the specified ledger_info
    pub fn new_with_signature(
        author: Author,
        ledger_info: LedgerInfo,
        signature: Ed25519Signature,
    ) -> Self {
        Self {
            author,
            ledger_info,
            signature,
        }
    }

    /// Return the author of the commit proposal
    pub fn author(&self) -> Author {
        self.author
    }

    /// Return the LedgerInfo associated with this commit proposal
    pub fn ledger_info(&self) -> &LedgerInfo {
        &self.ledger_info
    }

    /// Return the signature of the vote
    pub fn signature(&self) -> &Ed25519Signature {
        &self.signature
    }

    pub fn round(&self) -> Round { self.ledger_info.round() }

    pub fn epoch(&self) -> u64 { self.ledger_info.epoch() }

    /// Verifies that the consensus data hash of LedgerInfo corresponds to the vote info,
    /// and then verifies the signature.
    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {

        validator
            .verify(self.author(), &self.ledger_info, &self.signature)
            .context("Failed to verify Vote")?;

        Ok(())
    }
}
