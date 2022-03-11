use crate::{
    metrics::{COUNTER, TIMER},
    CommitQueue, DummyValue, Kv, ProofFetcher, Store,
};
use diem_crypto::HashValue;
use diem_infallible::Mutex;
use std::{
    collections::VecDeque,
    sync::{
        mpsc::{Receiver, SyncSender},
        Arc,
    },
    time::Instant,
};

const BATCH_SIZE: usize = 200_000;

pub fn phase_gen_payload(next_phase: SyncSender<Vec<Kv>>) {
    const BATCHES: usize = 1000;
    const BATCH_SIZE: usize = 200_000;

    for block in 0..BATCHES {
        let kvs = {
            let _timer = TIMER
                .with_label_values(&["phase_gen_payload"])
                .start_timer();

            let first_index = block * BATCH_SIZE + 1;
            (first_index..(first_index + BATCH_SIZE))
                .map(|i| {
                    let hash = HashValue::sha3_256_of(&i.to_le_bytes());
                    (hash, DummyValue(hash))
                })
                .collect()
        };
        next_phase.send(kvs).unwrap();
    }
}

pub fn phase_update(
    prev_phase: Receiver<Vec<Kv>>,
    commit_queue: Arc<Mutex<CommitQueue>>,
    next_phase: SyncSender<()>,
    proof_fetcher: ProofFetcher,
) {
    while let Ok(kvs) = prev_phase.recv() {
        let (version, smt) = commit_queue.lock().committed_version_and_latest_view();

        let (result_smt, nodes) = {
            let _timer = TIMER.with_label_values(&["phase_update"]).start_timer();
            let fsmt = smt.freeze();
            let proof_reader = proof_fetcher.fetch_proofs_for_missing(&fsmt, version, &kvs);
            let result_fsmt = {
                let _timer = TIMER.with_label_values(&["smt_update"]).start_timer();
                fsmt.batch_update(&kvs, &proof_reader).unwrap()
            };
            let nodes = {
                let _timer = TIMER.with_label_values(&["new_nodes_since"]).start_timer();
                result_fsmt.new_nodes_since(&fsmt)
            };
            (result_fsmt.unfreeze(), nodes)
        };

        commit_queue.lock().enqueue(result_smt, nodes);
        next_phase.send(()).unwrap();
    }
}

pub fn phase_commit(
    prev_phase: Receiver<()>,
    commit_queue: Arc<Mutex<CommitQueue>>,
    store: Arc<dyn Store>,
) {
    let mut num_updates = 0;
    let started = Instant::now();
    let mut smt_cache = VecDeque::from(vec![None; 63]);

    while prev_phase.recv().is_ok() {
        let (version, nodes) = commit_queue.lock().next_to_commit().unwrap();

        {
            let _timer = TIMER.with_label_values(&["phase_commit"]).start_timer();
            store.put_nodes(&nodes, version).unwrap();
            num_updates += BATCH_SIZE;
            COUNTER
                .with_label_values(&["num_updates"])
                .inc_by(BATCH_SIZE as u64);
            println!(
                "Committed. num_updates: {}, Accumulative TPS: {:.0}",
                num_updates,
                num_updates as f64 / started.elapsed().as_secs_f64()
            )
        }

        let stale_smt = commit_queue.lock().dequeue().unwrap();
        smt_cache.pop_front();
        smt_cache.push_back(Some(stale_smt));
    }
}
