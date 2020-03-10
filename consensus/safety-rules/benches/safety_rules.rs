// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::block::{block_test_utils, Block};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use libra_config::config::{OnDiskStorageConfig, SafetyRulesBackend};
use libra_secure_storage::{InMemoryStorage, OnDiskStorage};
use libra_types::crypto_proxies::ValidatorSigner;
use rand::Rng;
use safety_rules::{
    process_client_wrapper::ProcessClientWrapper, test_utils, PersistentSafetyStorage,
    SafetyRulesManager, TSafetyRules,
};
use tempfile::NamedTempFile;

/// Execute an in order series of blocks (0 <- 1 <- 2 <- 3 and commit 0 and continue to rotate
/// left, appending new blocks on the right, committing the left most block
fn lsr(mut safety_rules: Box<dyn TSafetyRules<Vec<u8>>>, signer: ValidatorSigner, n: u64) {
    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..2048).map(|_| rng.gen::<u8>()).collect();

    let genesis_block = Block::<Vec<u8>>::make_genesis_block();
    let genesis_qc = block_test_utils::certificate_for_genesis();
    let mut round = genesis_block.round();

    round += 1;
    let mut b0 = test_utils::make_proposal_with_qc(round, genesis_qc, &signer);
    safety_rules.update(b0.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&b0).unwrap();

    round += 1;
    let mut b1 = test_utils::make_proposal_with_parent(data.clone(), round, &b0, None, &signer);
    safety_rules.update(b1.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&b1).unwrap();

    round += 1;
    let mut b2 = test_utils::make_proposal_with_parent(data.clone(), round, &b1, None, &signer);
    safety_rules.update(b2.block().quorum_cert()).unwrap();
    safety_rules.construct_and_sign_vote(&b2).unwrap();

    for _i in 0..n {
        round += 1;
        let b3 =
            test_utils::make_proposal_with_parent(data.clone(), round, &b2, Some(&b0), &signer);

        safety_rules.update(b3.block().quorum_cert()).unwrap();
        safety_rules.construct_and_sign_vote(&b3).unwrap();

        b0 = b1;
        b1 = b2;
        b2 = b3;
    }
}

fn in_memory(n: u64) {
    let signer = ValidatorSigner::from_int(0);
    let storage = PersistentSafetyStorage::initialize(
        InMemoryStorage::new_boxed_in_memory_storage(),
        signer.private_key().clone(),
    );
    let safety_rules_manager = SafetyRulesManager::new_local(signer.author(), storage);
    lsr(safety_rules_manager.client(), signer, n);
}

fn on_disk(n: u64) {
    let signer = ValidatorSigner::from_int(0);
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let storage = PersistentSafetyStorage::initialize(
        OnDiskStorage::new_boxed_on_disk_storage(file_path),
        signer.private_key().clone(),
    );
    let safety_rules_manager = SafetyRulesManager::new_local(signer.author(), storage);
    lsr(safety_rules_manager.client(), signer, n);
}

fn serializer(n: u64) {
    let signer = ValidatorSigner::from_int(0);
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let storage = PersistentSafetyStorage::initialize(
        OnDiskStorage::new_boxed_on_disk_storage(file_path),
        signer.private_key().clone(),
    );
    let safety_rules_manager = SafetyRulesManager::new_serializer(signer.author(), storage);
    lsr(safety_rules_manager.client(), signer, n);
}

fn thread(n: u64) {
    let signer = ValidatorSigner::from_int(0);
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let storage = PersistentSafetyStorage::initialize(
        OnDiskStorage::new_boxed_on_disk_storage(file_path),
        signer.private_key().clone(),
    );
    let safety_rules_manager = SafetyRulesManager::new_thread(signer.author(), storage);
    lsr(safety_rules_manager.client(), signer, n);
}

fn process(n: u64) {
    let file_path = NamedTempFile::new().unwrap().into_temp_path().to_path_buf();
    let mut config = OnDiskStorageConfig::default();
    config.default = true;
    config.path = file_path;
    let backend = SafetyRulesBackend::OnDiskStorage(config);
    let client_wrapper = ProcessClientWrapper::new(backend);
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
