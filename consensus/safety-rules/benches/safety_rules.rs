// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::block::{block_test_utils, block_test_utils::random_payload, Block};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libra_config::config::{OnDiskStorageConfig, SecureBackend};
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_secure_storage::{InMemoryStorage, OnDiskStorage, Storage};
use libra_types::validator_signer::ValidatorSigner;
use safety_rules::{
    process_client_wrapper::ProcessClientWrapper, test_utils, PersistentSafetyStorage,
    SafetyRulesManager, TSafetyRules,
};
use tempfile::NamedTempFile;

/// Execute an in order series of blocks (0 <- 1 <- 2 <- 3 and commit 0 and continue to rotate
/// left, appending new blocks on the right, committing the left most block
fn lsr(mut safety_rules: Box<dyn TSafetyRules>, signer: ValidatorSigner, n: u64) {
    let data = random_payload(2048);

    let genesis_block = Block::make_genesis_block();
    let genesis_qc = block_test_utils::certificate_for_genesis();
    let mut round = genesis_block.round();

    round += 1;
    let mut b0 = test_utils::make_proposal_with_qc(round, genesis_qc, &signer, None);
    safety_rules.construct_and_sign_vote(&b0).unwrap();

    round += 1;
    let mut b1 =
        test_utils::make_proposal_with_parent(data.clone(), round, &b0, None, &signer, None);
    safety_rules.construct_and_sign_vote(&b1).unwrap();

    round += 1;
    let mut b2 =
        test_utils::make_proposal_with_parent(data.clone(), round, &b1, None, &signer, None);
    safety_rules.construct_and_sign_vote(&b2).unwrap();

    for _i in 0..n {
        round += 1;
        let b3 = test_utils::make_proposal_with_parent(
            data.clone(),
            round,
            &b2,
            Some(&b0),
            &signer,
            None,
        );

        safety_rules.construct_and_sign_vote(&b3).unwrap();

        b0 = b1;
        b1 = b2;
        b2 = b3;
    }
}

fn in_memory(n: u64) {
    let signer = ValidatorSigner::from_int(0);
    let waypoint = test_utils::validator_signers_to_waypoint(&[&signer]);
    let storage = PersistentSafetyStorage::initialize(
        Storage::from(InMemoryStorage::new()),
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
    );
    let safety_rules_manager = SafetyRulesManager::new_local(storage, false);
    lsr(safety_rules_manager.client(), signer, n);
}

fn on_disk(n: u64) {
    let signer = ValidatorSigner::from_int(0);
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let waypoint = test_utils::validator_signers_to_waypoint(&[&signer]);
    let storage = PersistentSafetyStorage::initialize(
        Storage::from(OnDiskStorage::new(file_path)),
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
    );
    let safety_rules_manager = SafetyRulesManager::new_local(storage, false);
    lsr(safety_rules_manager.client(), signer, n);
}

fn serializer(n: u64) {
    let signer = ValidatorSigner::from_int(0);
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let waypoint = test_utils::validator_signers_to_waypoint(&[&signer]);
    let storage = PersistentSafetyStorage::initialize(
        Storage::from(OnDiskStorage::new(file_path)),
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
    );
    let safety_rules_manager = SafetyRulesManager::new_serializer(storage, false);
    lsr(safety_rules_manager.client(), signer, n);
}

fn thread(n: u64) {
    let signer = ValidatorSigner::from_int(0);
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let waypoint = test_utils::validator_signers_to_waypoint(&[&signer]);
    let storage = PersistentSafetyStorage::initialize(
        Storage::from(OnDiskStorage::new(file_path)),
        signer.author(),
        signer.private_key().clone(),
        Ed25519PrivateKey::generate_for_testing(),
        waypoint,
    );
    let safety_rules_manager = SafetyRulesManager::new_thread(storage, false);
    lsr(safety_rules_manager.client(), signer, n);
}

fn process(n: u64) {
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let mut config = OnDiskStorageConfig::default();
    config.path = file_path;
    let backend = SecureBackend::OnDiskStorage(config);
    let client_wrapper = ProcessClientWrapper::new(backend, false);
    let signer = client_wrapper.signer();

    lsr(Box::new(client_wrapper), signer, n);
}

pub fn benchmark(c: &mut Criterion) {
    let count = 100;
    let mut group = c.benchmark_group("SafetyRules");
    group.bench_function("InMemory", |b| b.iter(|| in_memory(black_box(count))));
    group.bench_function("OnDisk", |b| b.iter(|| on_disk(black_box(count))));
    group.bench_function("Serializer", |b| b.iter(|| serializer(black_box(count))));
    group.bench_function("Thread", |b| b.iter(|| thread(black_box(count))));
    group.bench_function("Process", |b| b.iter(|| process(black_box(count))));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
