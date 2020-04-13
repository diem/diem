// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{safety_rules_manager, ConsensusState, Error, SafetyRulesManager, TSafetyRules};
use consensus_types::{
    block::Block,
    block_data::BlockData,
    common::{Payload, Round},
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote::Vote,
    vote_proposal::VoteProposal,
};
use libra_config::{
    config::{ConsensusType, NodeConfig, RemoteService, SafetyRulesBackend, SafetyRulesService},
    utils,
};
use libra_crypto::ed25519;
use libra_types::validator_signer::ValidatorSigner;
use std::{
    any::TypeId,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

/// This container exists only so that we can kill the spawned process after testing is complete.
/// Otherwise the process will be killed at the end of the safety_rules function and the test will
/// fail.
pub struct ProcessClientWrapper<T> {
    signer: ValidatorSigner,
    _safety_rules_manager: SafetyRulesManager<T>,
    safety_rules: Box<dyn TSafetyRules<T>>,
}

impl<T: Payload> ProcessClientWrapper<T> {
    pub fn new(backend: SafetyRulesBackend) -> Self {
        let server_port = utils::get_available_port();
        let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);

        let type_id = TypeId::of::<T>();
        let consensus_type = if type_id == TypeId::of::<Round>() {
            ConsensusType::Rounds
        } else if type_id == TypeId::of::<Vec<u8>>() {
            ConsensusType::Bytes
        } else {
            panic!("Invalid type: {:?}", type_id);
        };

        let remote_service = RemoteService {
            server_address,
            consensus_type,
        };
        let mut config = NodeConfig::random();
        config.consensus.safety_rules.backend = backend;
        config.consensus.safety_rules.service = SafetyRulesService::SpawnedProcess(remote_service);

        let mut test_config = config.test.as_ref().unwrap().clone();
        let safety_rules_manager = SafetyRulesManager::new(&mut config);
        let safety_rules = safety_rules_manager.client();
        let (author, _) = safety_rules_manager::extract_service_inputs(&mut config);
        let private_key = test_config
            .consensus_keypair
            .as_mut()
            .unwrap()
            .take_private()
            .unwrap();

        Self {
            signer: ValidatorSigner::new(author, private_key),
            _safety_rules_manager: safety_rules_manager,
            safety_rules,
        }
    }

    pub fn signer(&self) -> ValidatorSigner {
        self.signer.clone()
    }
}

impl<T: Payload> TSafetyRules<T> for ProcessClientWrapper<T> {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        self.safety_rules.consensus_state()
    }

    fn update(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        self.safety_rules.update(qc)
    }

    fn start_new_epoch(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        self.safety_rules.start_new_epoch(qc)
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal<T>) -> Result<Vote, Error> {
        self.safety_rules.construct_and_sign_vote(vote_proposal)
    }

    fn sign_proposal(&mut self, block_data: BlockData<T>) -> Result<Block<T>, Error> {
        self.safety_rules.sign_proposal(block_data)
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<ed25519::Signature, Error> {
        self.safety_rules.sign_timeout(timeout)
    }
}
