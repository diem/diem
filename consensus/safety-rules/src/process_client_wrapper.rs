// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils, ConsensusState, Error, SafetyRulesManager, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, quorum_cert::QuorumCert, timeout::Timeout, vote::Vote,
    vote_proposal::VoteProposal,
};
use libra_config::{
    config::{NodeConfig, RemoteService, SafetyRulesService, SecureBackend},
    utils,
};
use libra_crypto::ed25519::Ed25519Signature;
use libra_types::{epoch_change::EpochChangeProof, validator_signer::ValidatorSigner};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// This container exists only so that we can kill the spawned process after testing is complete.
/// Otherwise the process will be killed at the end of the safety_rules function and the test will
/// fail.
pub struct ProcessClientWrapper {
    signer: ValidatorSigner,
    _safety_rules_manager: SafetyRulesManager,
    safety_rules: Box<dyn TSafetyRules>,
}

impl ProcessClientWrapper {
    pub fn new(backend: SecureBackend) -> Self {
        let server_port = utils::get_available_port();
        let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);

        let remote_service = RemoteService { server_address };
        let mut config = NodeConfig::random();

        let mut test_config = config.test.as_ref().unwrap().clone();
        let author = config
            .validator_network
            .as_ref()
            .unwrap()
            .identity
            .peer_id_from_config()
            .unwrap();
        let private_key = test_config
            .consensus_keypair
            .as_mut()
            .unwrap()
            .take_private()
            .unwrap();
        let signer = ValidatorSigner::new(author, private_key);
        config.base.waypoint = Some(test_utils::validator_signers_to_waypoints(&[&signer]));

        config.consensus.safety_rules.backend = backend;
        config.consensus.safety_rules.service = SafetyRulesService::SpawnedProcess(remote_service);

        let safety_rules_manager = SafetyRulesManager::new(&mut config);
        let safety_rules = safety_rules_manager.client();

        Self {
            signer,
            _safety_rules_manager: safety_rules_manager,
            safety_rules,
        }
    }

    pub fn signer(&self) -> ValidatorSigner {
        self.signer.clone()
    }
}

impl TSafetyRules for ProcessClientWrapper {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        self.safety_rules.consensus_state()
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        self.safety_rules.initialize(proof)
    }

    fn update(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        self.safety_rules.update(qc)
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal) -> Result<Vote, Error> {
        self.safety_rules.construct_and_sign_vote(vote_proposal)
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        self.safety_rules.sign_proposal(block_data)
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        self.safety_rules.sign_timeout(timeout)
    }
}
