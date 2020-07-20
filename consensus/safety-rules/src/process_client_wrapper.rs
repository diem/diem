// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, ConsensusState, Error, SafetyRulesManager, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, timeout::Timeout, vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};
use libra_config::{
    config::{NodeConfig, RemoteService, SafetyRulesService, SecureBackend},
    utils,
};
use libra_crypto::ed25519::{Ed25519PrivateKey, Ed25519Signature};
use libra_types::{epoch_change::EpochChangeProof, validator_signer::ValidatorSigner};
use std::{
    marker::{Send, Sync},
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

/// This container exists only so that we can kill the spawned process after testing is complete.
/// Otherwise the process will be killed at the end of the safety_rules function and the test will
/// fail.
pub struct ProcessClientWrapper {
    signer: ValidatorSigner,
    _safety_rules_manager: SafetyRulesManager,
    safety_rules: Box<dyn TSafetyRules + Send + Sync>,
    execution_private_key: Option<Ed25519PrivateKey>,
}

impl ProcessClientWrapper {
    pub fn new(backend: SecureBackend, verify_vote_proposal_signature: bool) -> Self {
        let server_port = utils::get_available_port();
        let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port).into();

        let remote_service = RemoteService { server_address };
        let mut config = NodeConfig::random().consensus.safety_rules;
        let test_config = config.test.as_mut().unwrap();
        let author = test_config.author;
        let private_key = test_config.consensus_key.as_ref().unwrap().private_key();
        let signer = ValidatorSigner::new(author, private_key);
        let waypoint = test_utils::validator_signers_to_waypoint(&[&signer]);
        test_config.waypoint = Some(waypoint);

        config.backend = backend;
        config.verify_vote_proposal_signature = verify_vote_proposal_signature;
        config.service = SafetyRulesService::SpawnedProcess(remote_service);

        let execution_private_key = test_config
            .execution_key
            .as_ref()
            .map(|key| key.private_key());

        let safety_rules_manager = SafetyRulesManager::new(&config);
        let safety_rules = safety_rules_manager.client();

        Self {
            signer,
            _safety_rules_manager: safety_rules_manager,
            safety_rules,
            execution_private_key,
        }
    }

    pub fn signer(&self) -> ValidatorSigner {
        self.signer.clone()
    }

    pub fn execution_private_key(&mut self) -> Ed25519PrivateKey {
        self.execution_private_key.take().expect("must exist")
    }
}

impl TSafetyRules for ProcessClientWrapper {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        self.safety_rules.consensus_state()
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        self.safety_rules.initialize(proof)
    }

    fn construct_and_sign_vote(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        self.safety_rules.construct_and_sign_vote(vote_proposal)
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        self.safety_rules.sign_proposal(block_data)
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        self.safety_rules.sign_timeout(timeout)
    }
}
