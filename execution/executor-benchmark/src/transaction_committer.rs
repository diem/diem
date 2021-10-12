// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::hash::HashValue;
use diem_logger::prelude::*;
use diem_types::{
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    protocol_spec::DpnProto,
    transaction::Version,
};
use diem_vm::DiemVM;
use diemdb::metrics::DIEM_STORAGE_API_LATENCY_SECONDS;
use executor::{
    metrics::{
        DIEM_EXECUTOR_COMMIT_BLOCKS_SECONDS, DIEM_EXECUTOR_EXECUTE_BLOCK_SECONDS,
        DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS,
    },
    Executor,
};
use executor_types::BlockExecutor;
use std::{
    collections::BTreeMap,
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

pub(crate) fn gen_li_with_sigs(
    block_id: HashValue,
    root_hash: HashValue,
    version: Version,
) -> LedgerInfoWithSignatures {
    let block_info = BlockInfo::new(
        1,        /* epoch */
        0,        /* round, doesn't matter */
        block_id, /* id, doesn't matter */
        root_hash, version, 0,    /* timestamp_usecs, doesn't matter */
        None, /* next_epoch_state */
    );
    let ledger_info = LedgerInfo::new(
        block_info,
        HashValue::zero(), /* consensus_data_hash, doesn't matter */
    );
    LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new() /* signatures */)
}

pub struct TransactionCommitter {
    executor: Arc<Executor<DpnProto, DiemVM>>,
    start_version: Version,
    version: Version,
    block_receiver: mpsc::Receiver<(HashValue, HashValue, Instant, Instant, Duration, usize)>,
}

impl TransactionCommitter {
    pub fn new(
        executor: Arc<Executor<DpnProto, DiemVM>>,
        start_version: Version,
        block_receiver: mpsc::Receiver<(HashValue, HashValue, Instant, Instant, Duration, usize)>,
    ) -> Self {
        Self {
            start_version,
            version: start_version,
            executor,
            block_receiver,
        }
    }

    pub fn run(&mut self) {
        info!("Start with version: {}", self.version);

        while let Ok((
            block_id,
            root_hash,
            global_start_time,
            execution_start_time,
            execution_time,
            num_txns,
        )) = self.block_receiver.recv()
        {
            self.version += num_txns as u64;
            let commit_start = std::time::Instant::now();
            let ledger_info_with_sigs = gen_li_with_sigs(block_id, root_hash, self.version);
            self.executor
                .commit_blocks(vec![block_id], ledger_info_with_sigs)
                .unwrap();

            report_block(
                self.version - self.start_version,
                global_start_time,
                execution_start_time,
                execution_time,
                Instant::now().duration_since(commit_start),
                num_txns,
            );
        }
    }
}

fn report_block(
    version: Version,
    global_start_time: Instant,
    execution_start_time: Instant,
    execution_time: Duration,
    commit_time: Duration,
    block_size: usize,
) {
    info!(
        "Version: {}. latency: {} ms, execute time: {} ms. commit time: {} ms. TPS: {:.0}. Accumulative TPS: {:.0}",
        version,
        Instant::now().duration_since(execution_start_time).as_millis(),
        execution_time.as_millis(),
        commit_time.as_millis(),
        block_size as f64 / (std::cmp::max(execution_time, commit_time)).as_secs_f64(),
        version as f64 / global_start_time.elapsed().as_secs_f64(),
    );
    info!(
            "Accumulative total: VM time: {:.0} secs, executor time: {:.0} secs, commit time: {:.0} secs, DB commit time: {:.0} secs",
            DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS.get_sample_sum(),
            DIEM_EXECUTOR_EXECUTE_BLOCK_SECONDS.get_sample_sum() - DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS.get_sample_sum(),
            DIEM_EXECUTOR_COMMIT_BLOCKS_SECONDS.get_sample_sum(),
            DIEM_STORAGE_API_LATENCY_SECONDS.get_metric_with_label_values(&["save_transactions", "Ok"]).expect("must exist.").get_sample_sum(),
        );
    const NANOS_PER_SEC: f64 = 1_000_000_000.0;
    info!(
            "Accumulative per transaction: VM time: {:.0} ns, executor time: {:.0} ns, commit time: {:.0} ns, DB commit time: {:.0} ns",
            DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS.get_sample_sum() * NANOS_PER_SEC
                / version as f64,
            (DIEM_EXECUTOR_EXECUTE_BLOCK_SECONDS.get_sample_sum() - DIEM_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS.get_sample_sum()) * NANOS_PER_SEC
                / version as f64,
            DIEM_EXECUTOR_COMMIT_BLOCKS_SECONDS.get_sample_sum() * NANOS_PER_SEC
                / version as f64,
            DIEM_STORAGE_API_LATENCY_SECONDS.get_metric_with_label_values(&["save_transactions", "Ok"]).expect("must exist.").get_sample_sum() * NANOS_PER_SEC
                / version as f64,
        );
}
