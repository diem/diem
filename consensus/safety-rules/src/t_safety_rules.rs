// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, Error};
use consensus_types::{
    block_data::BlockData,
    timeout::Timeout,
    timeout_2chain::{TwoChainTimeout, TwoChainTimeoutCertificate},
    vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};
use diem_crypto::ed25519::Ed25519Signature;
use diem_types::{
    epoch_change::EpochChangeProof,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};

/// Interface for SafetyRules
pub trait TSafetyRules {
    /// Provides the internal state of SafetyRules for monitoring / debugging purposes. This does
    /// not include sensitive data like private keys.
    fn consensus_state(&mut self) -> Result<ConsensusState, Error>;

    /// Initialize SafetyRules using an Epoch ending LedgerInfo, this should map to what was
    /// provided in consensus_state. It will be used to initialize the ValidatorSet.
    /// This uses a EpochChangeProof because there's a possibility that consensus migrated to a
    /// new epoch but SafetyRules did not.
    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error>;

    /// Attempts to vote for a given proposal following the voting rules.
    fn construct_and_sign_vote(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error>;

    /// As the holder of the private key, SafetyRules also signs proposals or blocks.
    /// A Block is a signed BlockData along with some additional metadata.
    fn sign_proposal(&mut self, block_data: &BlockData) -> Result<Ed25519Signature, Error>;

    /// As the holder of the private key, SafetyRules also signs what is effectively a
    /// timeout message. This returns the signature for that timeout message.
    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error>;

    /// Sign the timeout together with highest qc for 2-chain protocol.
    fn sign_timeout_with_qc(
        &mut self,
        timeout: &TwoChainTimeout,
        timeout_cert: Option<&TwoChainTimeoutCertificate>,
    ) -> Result<Ed25519Signature, Error>;

    /// Sign the vote with 2-chain protocol.
    fn construct_and_sign_vote_two_chain(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
        timeout_cert: Option<&TwoChainTimeoutCertificate>,
    ) -> Result<Vote, Error>;

    /// As the holder of the private key, SafetyRules also signs a commit vote.
    /// This returns the signature for the commit vote.
    fn sign_commit_vote(
        &mut self,
        ledger_info: LedgerInfoWithSignatures,
        new_ledger_info: LedgerInfo,
    ) -> Result<Ed25519Signature, Error>;
}
