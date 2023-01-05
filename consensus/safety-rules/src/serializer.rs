// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, logging::LogEntry, ConsensusState, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block_data::BlockData,
    timeout::Timeout,
    timeout_2chain::{TwoChainTimeout, TwoChainTimeoutCertificate},
    vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};
use diem_crypto::ed25519::Ed25519Signature;
use diem_infallible::RwLock;
use diem_types::{
    epoch_change::EpochChangeProof,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SafetyRulesInput {
    ConsensusState,
    Initialize(Box<EpochChangeProof>),
    ConstructAndSignVote(Box<MaybeSignedVoteProposal>),
    SignProposal(Box<BlockData>),
    SignTimeout(Box<Timeout>),
    SignTimeoutWithQC(
        Box<TwoChainTimeout>,
        Box<Option<TwoChainTimeoutCertificate>>,
    ),
    ConstructAndSignVoteTwoChain(
        Box<MaybeSignedVoteProposal>,
        Box<Option<TwoChainTimeoutCertificate>>,
    ),
    SignCommitVote(Box<LedgerInfoWithSignatures>, Box<LedgerInfo>),
}

pub struct SerializerService {
    internal: SafetyRules,
}

impl SerializerService {
    pub fn new(internal: SafetyRules) -> Self {
        Self { internal }
    }

    pub fn handle_message(&mut self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = serde_json::from_slice(&input_message)?;

        let output = match input {
            SafetyRulesInput::ConsensusState => {
                serde_json::to_vec(&self.internal.consensus_state())
            }
            SafetyRulesInput::Initialize(li) => serde_json::to_vec(&self.internal.initialize(&li)),
            SafetyRulesInput::ConstructAndSignVote(vote_proposal) => {
                serde_json::to_vec(&self.internal.construct_and_sign_vote(&vote_proposal))
            }
            SafetyRulesInput::SignProposal(block_data) => {
                serde_json::to_vec(&self.internal.sign_proposal(&block_data))
            }
            SafetyRulesInput::SignTimeout(timeout) => {
                serde_json::to_vec(&self.internal.sign_timeout(&timeout))
            }
            SafetyRulesInput::SignTimeoutWithQC(timeout, maybe_tc) => serde_json::to_vec(
                &self
                    .internal
                    .sign_timeout_with_qc(&timeout, maybe_tc.as_ref().as_ref()),
            ),
            SafetyRulesInput::ConstructAndSignVoteTwoChain(vote_proposal, maybe_tc) => {
                serde_json::to_vec(
                    &self.internal.construct_and_sign_vote_two_chain(
                        &vote_proposal,
                        maybe_tc.as_ref().as_ref(),
                    ),
                )
            }
            SafetyRulesInput::SignCommitVote(ledger_info, new_ledger_info) => serde_json::to_vec(
                &self
                    .internal
                    .sign_commit_vote(*ledger_info, *new_ledger_info),
            ),
        };

        Ok(output?)
    }
}

pub struct SerializerClient {
    service: Box<dyn TSerializerClient>,
}

impl SerializerClient {
    pub fn new(serializer_service: Arc<RwLock<SerializerService>>) -> Self {
        let service = Box::new(LocalService { serializer_service });
        Self { service }
    }

    pub fn new_client(service: Box<dyn TSerializerClient>) -> Self {
        Self { service }
    }

    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        self.service.request(input)
    }
}

impl TSafetyRules for SerializerClient {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        let _timer = counters::start_timer("external", LogEntry::ConsensusState.as_str());
        let response = self.request(SafetyRulesInput::ConsensusState)?;
        serde_json::from_slice(&response)?
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let _timer = counters::start_timer("external", LogEntry::Initialize.as_str());
        let response = self.request(SafetyRulesInput::Initialize(Box::new(proof.clone())))?;
        serde_json::from_slice(&response)?
    }

    fn construct_and_sign_vote(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        let _timer = counters::start_timer("external", LogEntry::ConstructAndSignVote.as_str());
        let response = self.request(SafetyRulesInput::ConstructAndSignVote(Box::new(
            vote_proposal.clone(),
        )))?;
        serde_json::from_slice(&response)?
    }

    fn sign_proposal(&mut self, block_data: &BlockData) -> Result<Ed25519Signature, Error> {
        let _timer = counters::start_timer("external", LogEntry::SignProposal.as_str());
        let response =
            self.request(SafetyRulesInput::SignProposal(Box::new(block_data.clone())))?;
        serde_json::from_slice(&response)?
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        let _timer = counters::start_timer("external", LogEntry::SignTimeout.as_str());
        let response = self.request(SafetyRulesInput::SignTimeout(Box::new(timeout.clone())))?;
        serde_json::from_slice(&response)?
    }

    fn sign_timeout_with_qc(
        &mut self,
        timeout: &TwoChainTimeout,
        timeout_cert: Option<&TwoChainTimeoutCertificate>,
    ) -> Result<Ed25519Signature, Error> {
        let _timer = counters::start_timer("external", LogEntry::SignTimeoutWithQC.as_str());
        let response = self.request(SafetyRulesInput::SignTimeoutWithQC(
            Box::new(timeout.clone()),
            Box::new(timeout_cert.cloned()),
        ))?;
        serde_json::from_slice(&response)?
    }

    fn construct_and_sign_vote_two_chain(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
        timeout_cert: Option<&TwoChainTimeoutCertificate>,
    ) -> Result<Vote, Error> {
        let _timer =
            counters::start_timer("external", LogEntry::ConstructAndSignVoteTwoChain.as_str());
        let response = self.request(SafetyRulesInput::ConstructAndSignVoteTwoChain(
            Box::new(vote_proposal.clone()),
            Box::new(timeout_cert.cloned()),
        ))?;
        serde_json::from_slice(&response)?
    }

    fn sign_commit_vote(
        &mut self,
        ledger_info: LedgerInfoWithSignatures,
        new_ledger_info: LedgerInfo,
    ) -> Result<Ed25519Signature, Error> {
        let _timer = counters::start_timer("external", LogEntry::SignCommitVote.as_str());
        let response = self.request(SafetyRulesInput::SignCommitVote(
            Box::new(ledger_info),
            Box::new(new_ledger_info),
        ))?;
        serde_json::from_slice(&response)?
    }
}

pub trait TSerializerClient: Send + Sync {
    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error>;
}

struct LocalService {
    pub serializer_service: Arc<RwLock<SerializerService>>,
}

impl TSerializerClient for LocalService {
    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        let input_message = serde_json::to_vec(&input)?;
        self.serializer_service
            .write()
            .handle_message(input_message)
    }
}
