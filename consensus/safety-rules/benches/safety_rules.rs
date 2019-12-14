// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::{block::block_test_utils, block::Block};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libra_types::crypto_proxies::ValidatorSigner;
use rand::Rng;
use safety_rules::{
    test_utils, {InMemoryStorage, SafetyRules, TSafetyRules},
};
use std::sync::Arc;

/// Execute an in order series of blocks (0 <- 1 <- 2 <- 3 and commit 0 and continue to rotate
/// left, appending new blocks on the right, committing the left most block
fn lsr(n: u64) {
    let validator_signer = ValidatorSigner::from_int(0);
    let mut safety_rules = SafetyRules::new(
        InMemoryStorage::default_storage(),
        Arc::new(validator_signer.clone()),
    );

    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..2048).map(|_| rng.gen::<u8>()).collect();

    let genesis_block = Block::<Vec<u8>>::make_genesis_block();
    let genesis_qc = block_test_utils::certificate_for_genesis();
    let mut round = genesis_block.round();

    round += 1;
    let mut b0 = test_utils::make_proposal_with_qc(round, genesis_qc.clone(), &validator_signer);
    safety_rules.update(b0.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&b0).unwrap();

    round += 1;
    let mut b1 =
        test_utils::make_proposal_with_parent(data.clone(), round, &b0, None, &validator_signer);
    safety_rules.update(b1.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&b1).unwrap();

    round += 1;
    let mut b2 =
        test_utils::make_proposal_with_parent(data.clone(), round, &b1, None, &validator_signer);
    safety_rules.update(b2.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&b2).unwrap();

    for _i in 0..n {
        round += 1;
        let b3 = test_utils::make_proposal_with_parent(
            data.clone(),
            round,
            &b2,
            Some(&b0),
            &validator_signer,
        );

        safety_rules.update(b3.block().quorum_cert()).unwrap();
        safety_rules.construct_and_sign_vote(&b3).unwrap();

        b0 = b1;
        b1 = b2;
        b2 = b3;
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lsr 10", |b| b.iter(|| lsr(black_box(10))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
