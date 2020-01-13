// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    safety_rules_manager, tests::suite, ConsensusState, Error, SafetyRulesManager, TSafetyRules,
};
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
    config::{ConsensusType, NodeConfig, RemoteService, SafetyRulesService},
    utils,
};
use libra_types::crypto_proxies::{Signature, ValidatorSigner};
use std::{
    any::TypeId,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

#[test]
fn test() {
    suite::run_test_suite(safety_rules::<Round>, safety_rules::<Vec<u8>>);
}

fn safety_rules<T: Payload>() -> (Box<dyn TSafetyRules<T>>, Arc<ValidatorSigner>) {
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
    config.consensus.safety_rules.service = SafetyRulesService::SpawnedProcess(remote_service);

    let safety_rules_manager = SafetyRulesManager::new(&mut config);
    let safety_rules = safety_rules_manager.client();
    let client_wrapper = Box::new(ProcessClientWrapper {
        _safety_rules_manager: safety_rules_manager,
        safety_rules,
    });
    let (signer, _) = safety_rules_manager::extract_service_inputs(&mut config);
    (client_wrapper, Arc::new(signer))
}

// This container exists only so that we can kill the spawned process after testing is complete.
// Otherwise the process will be killed at the end of the safety_rules function and the test will
// fail.
struct ProcessClientWrapper<T> {
    _safety_rules_manager: SafetyRulesManager<T>,
    safety_rules: Box<dyn TSafetyRules<T>>,
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

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Signature, Error> {
        self.safety_rules.sign_timeout(timeout)
    }
}
