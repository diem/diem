// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    common::{Height, Round},
    consensus_types::{
        block::{Block, BlockSource},
        quorum_cert::QuorumCert,
    },
    test_utils::placeholder_certificate_for_block,
};

use crypto::HashValue;
use nextgen_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use proptest::{prelude::*, std_facade::hash_map::HashMap};
use std::{
    panic,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[cfg(test)]
use types::validator_signer::proptests;
use types::{validator_signer::ValidatorSigner, validator_verifier::ValidatorVerifier};

type LinearizedBlockForest<T> = Vec<Block<T>>;

prop_compose! {
    /// This strategy is a swiss-army tool to produce a low-level block
    /// dependent on signer, round, parent and ancestor_id.
    /// Note that the quorum certificate carried by this block is still placeholder: one will have
    /// to generate it later on when adding to the tree.
    pub fn make_block(
        _ancestor_id: HashValue,
        parent_id_strategy: impl Strategy<Value = HashValue>,
        round_strategy: impl Strategy<Value = Round>,
        height: Height,
        signer_strategy: impl Strategy<Value = ValidatorSigner<Ed25519PrivateKey>>,
    )(
        parent_id in parent_id_strategy,
        round in round_strategy,
        payload in 0usize..10usize,
        height in Just(height),
        signer in signer_strategy,
    ) -> Block<Vec<usize>> {
        Block::new_internal(
            vec![payload],
            parent_id,
            round,
            height,
            get_current_timestamp().as_micros() as u64,
            QuorumCert::certificate_for_genesis(),
            &signer,
        )
    }
}

/// This produces the genesis block
pub fn genesis_strategy() -> impl Strategy<Value = Block<Vec<usize>>> {
    Just(Block::make_genesis_block())
}

prop_compose! {
    /// This produces an unmoored block, with arbitrary parent & QC ancestor
    pub fn unmoored_block(ancestor_id_strategy: impl Strategy<Value = HashValue>)(
        ancestor_id in ancestor_id_strategy,
    )(
        block in make_block(
            ancestor_id,
            HashValue::arbitrary(),
            Round::arbitrary(),
            123,
            proptests::arb_signer(),
        )
    ) -> Block<Vec<usize>> {
        block
    }
}

/// Offers the genesis block.
pub fn leaf_strategy() -> impl Strategy<Value = Block<Vec<usize>>> {
    genesis_strategy().boxed()
}

prop_compose! {
    /// This produces a block with an invalid id (and therefore signature)
    /// given a valid block
    pub fn fake_id(block_strategy: impl Strategy<Value = Block<Vec<usize>>>)
        (fake_id in HashValue::arbitrary(),
         block in block_strategy) -> Block<Vec<usize>> {
            Block {
                timestamp_usecs: get_current_timestamp().as_micros() as u64,
                id: fake_id,
                payload: block.get_payload().clone(),
                round: block.round(),
                height: block.height(),
                parent_id: block.parent_id(),
                quorum_cert: block.quorum_cert().clone(),
                block_source: BlockSource::Proposal {
                    author: block.author().unwrap(),
                    signature: block.signature().unwrap().clone(),
                },
            }
        }
}

prop_compose! {
    fn bigger_round(initial_round: Round)(
        increment in 2..8,
        initial_round in Just(initial_round),
    ) -> Round {
        initial_round + increment as u64
    }
}

/// This produces a round that is often higher than the parent, but not
/// too high
pub fn some_round(initial_round: Round) -> impl Strategy<Value = Round> {
    prop_oneof![
        9 => Just(1 + initial_round),
        1 => bigger_round(initial_round),
    ]
}

prop_compose! {
    /// This creates a child with a parent on its left, and a QC on the left
    /// of the parent. This, depending on branching, does not require the
    /// QC to always be an ancestor or the parent to always be the highest QC
    fn child(
        signer_strategy: impl Strategy<Value = ValidatorSigner<Ed25519PrivateKey>>,
        block_forest_strategy: impl Strategy<Value = LinearizedBlockForest<Vec<usize>>>,
    )(
        signer in signer_strategy,
        (forest_vec, parent_idx, qc_idx) in block_forest_strategy
            .prop_flat_map(|forest_vec| {
                let len = forest_vec.len();
                (Just(forest_vec), 0..len)
            })
            .prop_flat_map(|(forest_vec, parent_idx)| {
                (Just(forest_vec), Just(parent_idx), 0..=parent_idx)
            }),
    )( block in make_block(
        // ancestor_id
        forest_vec[qc_idx].id(),
        // parent_id
        Just(forest_vec[parent_idx].id()),
        // round
        some_round(forest_vec[parent_idx].round()),
        // height,
        forest_vec[parent_idx].height() + 1,
        // signer
        Just(signer),
    ), mut forest in Just(forest_vec),
    ) -> LinearizedBlockForest<Vec<usize>> {
        forest.push(block);
        forest
    }
}

/// This creates a block forest with keys extracted from a specific
/// vector
fn block_forest_from_keys(
    depth: u32,
    keypairs: Vec<Ed25519PrivateKey>,
) -> impl Strategy<Value = LinearizedBlockForest<Vec<usize>>> {
    let leaf = leaf_strategy().prop_map(|block| vec![block]);
    // Note that having `expected_branch_size` of 1 seems to generate significantly larger trees
    // than desired (this is my understanding after reading the documentation:
    // https://docs.rs/proptest/0.3.0/proptest/strategy/trait.Strategy.html#method.prop_recursive)
    leaf.prop_recursive(depth, depth, 2, move |inner| {
        child(proptests::mostly_in_keypair_pool(keypairs.clone()), inner)
    })
}

/// This returns keys and a block forest created from them
pub fn block_forest_and_its_keys(
    quorum_size: usize,
    depth: u32,
) -> impl Strategy<Value = (Vec<Ed25519PrivateKey>, LinearizedBlockForest<Vec<usize>>)> {
    proptest::collection::vec(proptests::arb_signing_key(), quorum_size).prop_flat_map(
        move |private_key| {
            (
                Just(private_key.clone()),
                block_forest_from_keys(depth, private_key),
            )
        },
    )
}

#[test]
fn test_genesis() {
    // Test genesis and the next block
    let genesis_block = Block::<i64>::make_genesis_block();
    assert_eq!(genesis_block.height(), 0);
    assert_eq!(genesis_block.parent_id(), HashValue::zero());
    assert_ne!(genesis_block.id(), HashValue::zero());
    assert!(genesis_block.is_genesis_block());
}

#[test]
fn test_nil_block() {
    let genesis_block = Block::make_genesis_block();
    let quorum_cert = QuorumCert::certificate_for_genesis();

    let nil_block = Block::make_nil_block(&genesis_block, 1, quorum_cert);
    assert_eq!(
        nil_block.quorum_cert().certified_block_id(),
        genesis_block.id()
    );
    assert_eq!(nil_block.round(), 1);
    assert_eq!(nil_block.timestamp_usecs(), genesis_block.timestamp_usecs());
    assert_eq!(nil_block.is_nil_block(), true);
    assert!(nil_block.author().is_none());

    let dummy_verifier = Arc::new(ValidatorVerifier::<Ed25519PublicKey>::new(HashMap::new()));
    assert!(nil_block.verify(dummy_verifier.as_ref()).is_ok());

    let signer = ValidatorSigner::random(None);
    let payload = 101;
    let nil_block_qc = placeholder_certificate_for_block(
        vec![&signer],
        nil_block.id(),
        nil_block.round(),
        nil_block.quorum_cert().certified_block_id(),
        nil_block.quorum_cert().certified_block_round(),
        nil_block.quorum_cert().certified_parent_block_id(),
        nil_block.quorum_cert().certified_parent_block_round(),
    );
    let nil_block_child = Block::make_block(
        &nil_block,
        payload,
        2,
        get_current_timestamp().as_micros() as u64,
        nil_block_qc.clone(),
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
    let quorum_cert = QuorumCert::certificate_for_genesis();
    let payload = 101;
    let next_block = Block::make_block(
        &genesis_block,
        payload,
        1,
        get_current_timestamp().as_micros() as u64,
        quorum_cert,
        &signer,
    );
    assert_eq!(next_block.round(), 1);
    assert_eq!(next_block.height(), 1);
    assert_eq!(genesis_block.is_parent_of(&next_block), true);
    assert_eq!(
        next_block.quorum_cert().certified_block_id(),
        genesis_block.id()
    );
    assert_eq!(next_block.get_payload(), &payload);

    let cloned_block = next_block.clone();
    assert_eq!(cloned_block.round(), next_block.round());
}

#[test]
fn test_block_qc() {
    // Verify that it's impossible to create a block with QC that doesn't point to a parent.
    let signer = ValidatorSigner::random(None);
    // Test genesis and the next block
    let genesis_block = Block::make_genesis_block();
    let genesis_qc = QuorumCert::certificate_for_genesis();

    let payload = 42;
    let a1 = Block::make_block(
        &genesis_block,
        payload,
        1,
        get_current_timestamp().as_micros() as u64,
        genesis_qc.clone(),
        &signer,
    );
    let a1_qc = placeholder_certificate_for_block(
        vec![&signer],
        a1.id(),
        a1.round(),
        a1.quorum_cert().certified_block_id(),
        a1.quorum_cert().certified_block_round(),
        a1.quorum_cert().certified_parent_block_id(),
        a1.quorum_cert().certified_parent_block_round(),
    );

    let result = panic::catch_unwind(|| {
        // should panic because qc does not point to parent
        Block::make_block(
            &a1,
            payload,
            2,
            get_current_timestamp().as_micros() as u64,
            genesis_qc.clone(),
            &signer,
        );
    });
    assert!(result.is_err());

    // once qc is correct, should not panic
    let a2 = Block::make_block(
        &a1,
        payload,
        2,
        get_current_timestamp().as_micros() as u64,
        a1_qc.clone(),
        &signer,
    );
    assert_eq!(a2.height(), 2);
}

// Using current_timestamp in this test
// because it's a bit hard to generate incremental timestamps in proptests
fn get_current_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Timestamp generated is before the UNIX_EPOCH!")
}
