// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{metrics_safety_rules::MetricsSafetyRules, test_utils::MockStorage};
use consensus_types::{
    block::block_test_utils::certificate_for_genesis, common::Round, executed_block::ExecutedBlock,
    quorum_cert::QuorumCert, vote_proposal::MaybeSignedVoteProposal,
};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519Signature},
    hash::ACCUMULATOR_PLACEHOLDER_HASH,
    HashValue, Uniform,
};
use diem_infallible::Mutex;
use diem_secure_storage::Storage;
use diem_types::{
    account_address::AccountAddress,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_signer::ValidatorSigner,
    validator_verifier::random_validator_verifier,
    waypoint::Waypoint,
};
use executor_types::StateComputeResult;
use safety_rules::{
    test_utils::{make_proposal_with_parent, make_proposal_with_qc},
    PersistentSafetyStorage, SafetyRulesManager,
};
use std::{collections::BTreeMap, sync::Arc};

pub fn prepare_safety_rules() -> (Arc<Mutex<MetricsSafetyRules>>, Vec<ValidatorSigner>) {
    let num_nodes = 1;

    // environment setup
    let (signers, validators) = random_validator_verifier(num_nodes, None, false);
    let validator_set = (&validators).into();
    let signer = &signers[0];

    let waypoint =
        Waypoint::new_epoch_boundary(&LedgerInfo::mock_genesis(Some(validator_set))).unwrap();

    let safety_storage = PersistentSafetyStorage::initialize(
        Storage::from(diem_secure_storage::InMemoryStorage::new()),
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
        true,
    );
    let (_, storage) = MockStorage::start_for_testing((&validators).into());

    let safety_rules_manager = SafetyRulesManager::new_local(safety_storage, false, false);
    let mut safety_rules = MetricsSafetyRules::new(safety_rules_manager.client(), storage);
    safety_rules.perform_initialize().unwrap();

    (Arc::new(Mutex::new(safety_rules)), signers)
}

// This function priorizes using parent over init_qc
pub fn prepare_executed_blocks_with_ledger_info(
    signer: &ValidatorSigner,
    num_blocks: Round,
    executed_hash: HashValue,
    consensus_hash: HashValue,
    some_parent: Option<MaybeSignedVoteProposal>,
    init_qc: Option<QuorumCert>,
    init_round: Round,
) -> (
    Vec<ExecutedBlock>,
    LedgerInfoWithSignatures,
    Vec<MaybeSignedVoteProposal>,
) {
    assert!(num_blocks > 0);

    let p1 = if let Some(parent) = some_parent {
        make_proposal_with_parent(vec![], init_round, &parent, None, signer, None)
    } else {
        make_proposal_with_qc(init_round, init_qc.unwrap(), signer, None)
    };

    let mut proposals: Vec<MaybeSignedVoteProposal> = vec![p1];

    for i in 1..num_blocks {
        println!("Generating {}", i);
        let parent = proposals.last().unwrap();
        let proposal =
            make_proposal_with_parent(vec![], init_round + i, parent, None, signer, None);
        proposals.push(proposal);
    }

    let compute_result = StateComputeResult::new(
        executed_hash,
        vec![], // dummy subtree
        0,
        vec![],
        0,
        None,
        vec![],
        vec![],
        vec![],
    );

    let li = LedgerInfo::new(
        proposals.last().unwrap().block().gen_block_info(
            compute_result.root_hash(),
            compute_result.version(),
            compute_result.epoch_state().clone(),
        ),
        consensus_hash,
    );

    let mut li_sig = LedgerInfoWithSignatures::new(
        li.clone(),
        BTreeMap::<AccountAddress, Ed25519Signature>::new(),
    );

    li_sig.add_signature(signer.author(), signer.sign(&li));

    let executed_blocks: Vec<ExecutedBlock> = proposals
        .iter()
        .map(|proposal| ExecutedBlock::new(proposal.block().clone(), compute_result.clone()))
        .collect();

    (executed_blocks, li_sig, proposals)
}

pub fn prepare_executed_blocks_with_executed_ledger_info(
    signer: &ValidatorSigner,
) -> (Vec<ExecutedBlock>, LedgerInfoWithSignatures) {
    let genesis_qc = certificate_for_genesis();
    let (executed_blocks, li_sig, _) = prepare_executed_blocks_with_ledger_info(
        signer,
        1,
        HashValue::random(),
        HashValue::from_u64(0xbeef),
        None,
        Some(genesis_qc),
        0,
    );
    (executed_blocks, li_sig)
}

pub fn prepare_executed_blocks_with_ordered_ledger_info(
    signer: &ValidatorSigner,
) -> (Vec<ExecutedBlock>, LedgerInfoWithSignatures) {
    let genesis_qc = certificate_for_genesis();
    let (executed_blocks, li_sig, _) = prepare_executed_blocks_with_ledger_info(
        signer,
        1,
        *ACCUMULATOR_PLACEHOLDER_HASH,
        *ACCUMULATOR_PLACEHOLDER_HASH,
        None,
        Some(genesis_qc),
        0,
    );
    (executed_blocks, li_sig)
}
