// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block::{
        block_test_utils::{certificate_for_genesis, *},
        Block,
    },
    quorum_cert::QuorumCert,
};
use libra_crypto::hash::{CryptoHash, HashValue};
use libra_types::{validator_signer::ValidatorSigner, validator_verifier::ValidatorVerifier};
use std::{collections::BTreeMap, panic, sync::Arc};

#[test]
fn test_genesis() {
    // Test genesis and the next block
    let genesis_block = Block::<i16>::make_genesis_block();
    assert_eq!(genesis_block.parent_id(), HashValue::zero());
    assert_ne!(genesis_block.id(), HashValue::zero());
    assert!(genesis_block.is_genesis_block());
}

#[test]
fn test_nil_block() {
    let genesis_block = Block::<i16>::make_genesis_block();
    let quorum_cert = certificate_for_genesis();

    let nil_block = Block::<i16>::new_nil(1, quorum_cert);
    assert_eq!(
        nil_block.quorum_cert().certified_block().id(),
        genesis_block.id()
    );
    assert_eq!(nil_block.round(), 1);
    assert_eq!(nil_block.timestamp_usecs(), genesis_block.timestamp_usecs());
    assert_eq!(nil_block.is_nil_block(), true);
    assert!(nil_block.author().is_none());

    let dummy_verifier = Arc::new(ValidatorVerifier::new(BTreeMap::new()));
    assert!(nil_block
        .validate_signatures(dummy_verifier.as_ref())
        .is_ok());
    assert!(nil_block.verify_well_formed().is_ok());

    let signer = ValidatorSigner::random(None);
    let payload = 101;
    let parent_block_info = nil_block.quorum_cert().certified_block();
    let nil_block_qc = gen_test_certificate(
        vec![&signer],
        nil_block.gen_block_info(
            parent_block_info.executed_state_id(),
            parent_block_info.version(),
            parent_block_info.next_epoch_info().cloned(),
        ),
        nil_block.quorum_cert().certified_block().clone(),
        None,
    );
    println!(
        "{:?} {:?}",
        nil_block.id(),
        nil_block_qc.certified_block().id()
    );
    let nil_block_child = Block::new_proposal(
        payload,
        2,
        get_current_timestamp().as_micros() as u64,
        nil_block_qc,
        &signer,
    );
    assert_eq!(nil_block_child.is_nil_block(), false);
    assert_eq!(nil_block_child.round(), 2);
    assert_eq!(nil_block_child.parent_id(), nil_block.id());
}

#[test]
fn test_block_relation() {
    let signer = ValidatorSigner::random(None);
    // Test genesis and the next block
    let genesis_block = Block::make_genesis_block();
    let quorum_cert = certificate_for_genesis();
    let payload = 101;
    let next_block = Block::new_proposal(
        payload,
        1,
        get_current_timestamp().as_micros() as u64,
        quorum_cert,
        &signer,
    );
    assert_eq!(next_block.round(), 1);
    assert_eq!(genesis_block.is_parent_of(&next_block), true);
    assert_eq!(
        next_block.quorum_cert().certified_block().id(),
        genesis_block.id()
    );
    assert_eq!(next_block.payload(), Some(&payload));

    let cloned_block = next_block.clone();
    assert_eq!(cloned_block.round(), next_block.round());
}

// Ensure that blocks that extend from the same QuorumCertificate but with different signatures
// have different block ids.
#[test]
fn test_same_qc_different_authors() {
    let signer = ValidatorSigner::random(None);
    let genesis_qc = certificate_for_genesis();
    let round = 1;
    let payload = 42;
    let current_timestamp = get_current_timestamp().as_micros() as u64;
    let block_round_1 = Block::new_proposal(
        payload,
        round,
        current_timestamp,
        genesis_qc.clone(),
        &signer,
    );

    let signature = signer.sign_message(genesis_qc.ledger_info().ledger_info().hash());
    let mut ledger_info_altered = genesis_qc.ledger_info().clone();
    ledger_info_altered.add_signature(signer.author(), signature);
    let genesis_qc_altered = QuorumCert::new(genesis_qc.vote_data().clone(), ledger_info_altered);

    let block_round_1_altered = Block::new_proposal(
        payload,
        round,
        current_timestamp,
        genesis_qc_altered,
        &signer,
    );

    let block_round_1_same =
        Block::new_proposal(payload, round, current_timestamp, genesis_qc, &signer);

    assert!(block_round_1.id() != block_round_1_altered.id());
    assert_eq!(block_round_1.id(), block_round_1_same.id());
}
