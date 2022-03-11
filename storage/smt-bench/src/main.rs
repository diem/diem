// Copyright (c) The Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    components::{
        commit_queue::CommitQueue, proof_fetcher::ProofFetcher, rocksdb_store::RocksDbStore,
    },
    interface::Store,
    phases::{phase_commit, phase_gen_payload, phase_update},
    types::{DummyValue, Kv, Smt},
};
use diem_infallible::Mutex;
use diem_secure_push_metrics::MetricsPusher;
use rayon::ThreadPoolBuilder;
use std::{
    sync::{mpsc::sync_channel, Arc},
    thread,
};

mod components;
mod interface;
mod metrics;
mod phases;
mod types;

fn main() {
    let _mp = MetricsPusher::start();

    let io_threads = 128;
    let cpu_threads = num_cpus::get();
    let io_pool = Arc::new(
        ThreadPoolBuilder::new()
            .thread_name(|n| format!("io-{}", n))
            .num_threads(io_threads)
            .build()
            .unwrap(),
    );
    let io_pool_clone = io_pool.clone();
    let cpu_pool = ThreadPoolBuilder::new()
        .thread_name(|n| format!("cpu-{}", n))
        .num_threads(cpu_threads)
        .build()
        .unwrap();

    let store = Arc::new(RocksDbStore::open(&"/home/msmouse/work/data/tmp").unwrap());
    let proof_fetcher = ProofFetcher::new(store.clone());
    let commit_queue = Arc::new(Mutex::new(CommitQueue::new(0, Smt::new_empty())));
    let commit_queue_clone = commit_queue.clone();

    let (kvs_tx, kvs_rx) = sync_channel(3 /* bound */);
    let (commit_tx, commit_rx) = sync_channel(3 /* bound */);

    let threads = vec![
        thread::Builder::new()
            .spawn(move || io_pool.install(move || phase_gen_payload(kvs_tx)))
            .unwrap(),
        thread::Builder::new()
            .spawn(move || {
                cpu_pool
                    .install(move || phase_update(kvs_rx, commit_queue, commit_tx, proof_fetcher))
            })
            .unwrap(),
        thread::Builder::new()
            .spawn(move || {
                io_pool_clone.install(move || phase_commit(commit_rx, commit_queue_clone, store))
            })
            .unwrap(),
    ];

    for t in threads {
        t.join().unwrap();
    }
}
