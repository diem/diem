// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{common::Author, quorum_cert::QuorumCert};
use anyhow::{ensure, Context};
use diem_crypto::ed25519::Ed25519Signature;
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use diem_types::{
    block_info::Round, validator_signer::ValidatorSigner, validator_verifier::ValidatorVerifier,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

/// This structure contains all the information necessary to construct a signature
/// on the equivalent of a DiemBFT v4 timeout message.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TwoChainTimeout {
    /// Epoch number corresponds to the set of validators that are active for this round.
    epoch: u64,
    /// The consensus protocol executes proposals (blocks) in rounds, which monotonically increase per epoch.
    round: Round,
    /// The highest quorum cert the signer has seen.
    quorum_cert: QuorumCert,
}

impl TwoChainTimeout {
    pub fn new(epoch: u64, round: Round, quorum_cert: QuorumCert) -> Self {
        Self {
            epoch,
            round,
            quorum_cert,
        }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn hqc_round(&self) -> Round {
        self.quorum_cert.certified_block().round()
    }

    pub fn quorum_cert(&self) -> &QuorumCert {
        &self.quorum_cert
    }

    pub fn sign(&self, signer: &ValidatorSigner) -> Ed25519Signature {
        signer.sign(&self.signing_format())
    }

    pub fn signing_format(&self) -> TimeoutSigningRepr {
        TimeoutSigningRepr {
            epoch: self.epoch(),
            round: self.round(),
            hqc_round: self.hqc_round(),
        }
    }
}

impl Display for TwoChainTimeout {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Timeout: [epoch: {}, round: {}, hqc_round: {}]",
            self.epoch,
            self.round,
            self.hqc_round(),
        )
    }
}

/// Validators sign this structure that allows the TwoChainTimeoutCertificate to store a round number
/// instead of a quorum cert per validator in the signatures field.
#[derive(Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
pub struct TimeoutSigningRepr {
    pub epoch: u64,
    pub round: Round,
    pub hqc_round: Round,
}

/// TimeoutCertificate is a proof that 2f+1 participants in epoch i
/// have voted in round r and we can now move to round r+1. DiemBFT v4 requires signature to sign on
/// the TimeoutSigningRepr and carry the TimeoutWithHighestQC with highest quorum cert among 2f+1.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TwoChainTimeoutCertificate {
    timeout: TwoChainTimeout,
    signatures: BTreeMap<Author, (Round, Ed25519Signature)>,
}

impl Display for TwoChainTimeoutCertificate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "TimeoutCertificate[epoch: {}, round: {}, hqc_round: {}]",
            self.timeout.epoch(),
            self.timeout.round(),
            self.timeout.hqc_round(),
        )
    }
}

impl TwoChainTimeoutCertificate {
    /// Creates new TimeoutCertificate
    pub fn new(timeout: TwoChainTimeout) -> Self {
        Self {
            timeout,
            signatures: BTreeMap::new(),
        }
    }
    /// Verifies the signatures for each validator, the signature is on the TimeoutSigningRepr where the
    /// hqc_round is in the signature map.
    /// We verify the following:
    /// 1. the highest quorum cert is valid
    /// 2. all signatures are properly formed (timeout.epoch, timeout.round, round)
    /// 3. timeout.hqc_round == max(signed round)
    pub fn verify(&self, validators: &ValidatorVerifier) -> anyhow::Result<()> {
        // Verify the highest quorum cert validity.
        let hqc_round = self.timeout.hqc_round();
        ensure!(
            hqc_round < self.timeout.round(),
            "Timeout round should be larger than the QC round"
        );
        self.timeout.quorum_cert().verify(validators)?;
        let mut signed_round = 0;
        validators.check_voting_power(self.signatures.keys())?;
        for (author, (qc_round, signature)) in &self.signatures {
            let t = TimeoutSigningRepr {
                epoch: self.timeout.epoch(),
                round: self.timeout.round(),
                hqc_round: *qc_round,
            };
            validators
                .verify(*author, &t, signature)
                .with_context(|| format!("Failed to verify {}'s TimeoutSigningRepr", *author))?;
            signed_round = std::cmp::max(signed_round, *qc_round);
        }
        ensure!(
            hqc_round == signed_round,
            "Inconsistent hqc round, qc has round {}, highest signed round {}",
            hqc_round,
            signed_round
        );
        Ok(())
    }

    /// The epoch of the timeout.
    pub fn epoch(&self) -> u64 {
        self.timeout.epoch()
    }

    /// The round of the timeout.
    pub fn round(&self) -> Round {
        self.timeout.round()
    }

    /// The highest hqc round of the 2f+1 participants
    pub fn highest_hqc_round(&self) -> Round {
        self.timeout.hqc_round()
    }

    /// Returns the signatures certifying the round
    pub fn signers(&self) -> impl Iterator<Item = &Author> {
        self.signatures.iter().map(|(k, _)| k)
    }

    /// Add a new timeout message from author, the timeout should already be verified in upper layer.
    pub fn add(&mut self, author: Author, timeout: TwoChainTimeout, signature: Ed25519Signature) {
        debug_assert_eq!(
            self.timeout.epoch(),
            timeout.epoch(),
            "Timeout should have the same epoch as TimeoutCert"
        );
        debug_assert_eq!(
            self.timeout.round(),
            timeout.round(),
            "Timeout should have the same round as TimeoutCert"
        );
        let hqc_round = timeout.hqc_round();
        if timeout.hqc_round() > self.timeout.hqc_round() {
            self.timeout = timeout;
        }
        self.signatures.insert(author, (hqc_round, signature));
    }
}

#[test]
fn test_2chain_timeout_certificate() {
    use crate::vote_data::VoteData;
    use diem_crypto::hash::CryptoHash;
    use diem_types::{
        block_info::BlockInfo,
        ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
        validator_verifier::random_validator_verifier,
    };

    let num_nodes = 4;
    let (signers, validators) = random_validator_verifier(num_nodes, None, false);
    let quorum_size = validators.quorum_voting_power() as usize;
    let generate_quorum = |round, num_of_signature| {
        let vote_data = VoteData::new(BlockInfo::random(round), BlockInfo::random(0));
        let mut ledger_info = LedgerInfoWithSignatures::new(
            LedgerInfo::new(BlockInfo::empty(), vote_data.hash()),
            BTreeMap::new(),
        );
        for signer in &signers[0..num_of_signature] {
            let signature = signer.sign(ledger_info.ledger_info());
            ledger_info.add_signature(signer.author(), signature);
        }
        QuorumCert::new(vote_data, ledger_info)
    };
    let generate_timeout =
        |round, qc_round| TwoChainTimeout::new(1, round, generate_quorum(qc_round, quorum_size));

    let timeouts: Vec<_> = (1..=3)
        .map(|qc_round| generate_timeout(4, qc_round))
        .collect();
    // timeout cert with (round, hqc round) = (4, 1), (4, 2), (4, 3)
    let mut valid_timeout_cert = TwoChainTimeoutCertificate::new(timeouts[0].clone());
    for (timeout, signer) in timeouts.iter().zip(&signers) {
        valid_timeout_cert.add(signer.author(), timeout.clone(), timeout.sign(signer));
    }
    valid_timeout_cert.verify(&validators).unwrap();

    // timeout round < hqc round
    let mut invalid_timeout_cert = valid_timeout_cert.clone();
    invalid_timeout_cert.timeout.round = 1;
    invalid_timeout_cert.verify(&validators).unwrap_err();

    // invalid signature
    let mut invalid_timeout_cert = valid_timeout_cert.clone();
    invalid_timeout_cert
        .signatures
        .get_mut(&signers[0].author())
        .unwrap()
        .1 = Ed25519Signature::dummy_signature();
    invalid_timeout_cert.verify(&validators).unwrap_err();

    // not enough signatures
    let mut invalid_timeout_cert = valid_timeout_cert.clone();
    invalid_timeout_cert
        .signatures
        .remove(&signers[0].author())
        .unwrap();
    invalid_timeout_cert.verify(&validators).unwrap_err();

    // hqc round does not match signed round
    let mut invalid_timeout_cert = valid_timeout_cert.clone();
    invalid_timeout_cert.timeout.quorum_cert = generate_quorum(2, quorum_size);
    invalid_timeout_cert.verify(&validators).unwrap_err();

    // invalid quorum cert
    let mut invalid_timeout_cert = valid_timeout_cert;
    invalid_timeout_cert.timeout.quorum_cert = generate_quorum(3, quorum_size - 1);
    invalid_timeout_cert.verify(&validators).unwrap_err();
}
