// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_config::config::RocksdbConfig;
use diem_types::{account_address::HashAccountAddress, account_state_blob::AccountStateBlob};
use diemdb::DiemDB;
use diemdb_benchmark::{gen_account_from_index, gen_random_blob};
use executor_types::ProofReader;
use rand::Rng;
use std::{collections::HashMap, path::PathBuf};
use storage_interface::DbReader;

type SparseMerkleTree = scratchpad::SparseMerkleTree<AccountStateBlob>;

pub fn run_benchmark(num_updates: usize, max_accounts: u64, blob_size: usize, db_dir: PathBuf) {
    let db = DiemDB::open(
        &db_dir,
        false, /* readonly */
        None,  /* pruner */
        RocksdbConfig::default(),
    )
    .expect("DB should open.");

    let mut rng = ::rand::thread_rng();

    let updates = (0..num_updates)
        .into_iter()
        .map(|_| {
            (
                gen_account_from_index(rng.gen_range(0..max_accounts)),
                gen_random_blob(blob_size, &mut rng),
            )
        })
        .collect::<Vec<_>>();

    let version = db.get_latest_version().unwrap();
    let account_state_proofs = updates
        .iter()
        .map(|(k, _)| {
            db.get_account_state_with_proof(*k, version, version)
                .map(|p| p.proof.transaction_info_to_account_proof().clone())
        })
        .collect::<Result<Vec<_>>>()
        .unwrap();

    let proof_reader = ProofReader::new(
        itertools::zip_eq(
            updates.iter().map(|(k, _)| k.hash()),
            account_state_proofs.into_iter(),
        )
        .collect::<HashMap<_, _>>(),
    );
    let root = db.get_latest_state_root().unwrap().1;
    let smt = SparseMerkleTree::new(root);
    let start = std::time::Instant::now();
    smt.batch_update(
        updates
            .iter()
            .map(|(k, v)| (k.hash(), v))
            .collect::<Vec<_>>(),
        &proof_reader,
    )
    .unwrap();
    println!(
        "Sparse Merkle Tree batch update {} updates: {}ms",
        num_updates,
        start.elapsed().as_millis()
    );
}
