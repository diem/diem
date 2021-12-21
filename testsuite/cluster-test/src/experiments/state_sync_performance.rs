// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashSet, fmt, time::Duration};

use diem_sdk::transaction_builder::TransactionFactory;
use forge::TxnEmitter;
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use structopt::StructOpt;
use tokio::time;

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance::Instance,
};
use async_trait::async_trait;
use diem_logger::info;
use std::time::Instant;

const EXPERIMENT_DURATION_TIMEOUT_SECS: u64 = 1000;
const STATE_SYNC_COMMITTED_COUNTER_NAME: &str = "diem_state_sync_version.synced";

#[derive(StructOpt, Debug)]
pub struct StateSyncPerformanceParams {
    emit_transactions_duration_secs: u64,
}

impl StateSyncPerformanceParams {
    pub fn new(emit_transactions_duration_secs: u64) -> Self {
        Self {
            emit_transactions_duration_secs,
        }
    }
}

pub struct StateSyncPerformance {
    params: StateSyncPerformanceParams,
    fullnode_instance: Instance,
    validator_instance: Instance,
}

impl ExperimentParam for StateSyncPerformanceParams {
    type E = StateSyncPerformance;

    fn build(self, cluster: &Cluster) -> Self::E {
        let validator_instance = cluster.random_validator_instance();
        let fullnode_instance = cluster.random_fullnode_instance();
        Self::E {
            params: self,
            fullnode_instance,
            validator_instance,
        }
    }
}

#[async_trait]
impl Experiment for StateSyncPerformance {
    fn affected_validators(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        result.insert(self.validator_instance.peer_name().clone());
        result
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );

        // Stop the fullnode and clear all data so that it falls behind
        info!("Stopping the fullnode: {}", self.fullnode_instance);
        self.fullnode_instance.stop().await?;
        self.fullnode_instance.clean_data().await?;

        // Execute and commit transactions on the validators for the specified duration
        let emit_transactions_duration_secs = self.params.emit_transactions_duration_secs;
        info!(
            "Executing transactions for {} seconds",
            emit_transactions_duration_secs
        );
        let emit_job_request = crate::util::emit_job_request_for_instances(
            context.cluster.validator_instances().to_vec(),
            context.global_emit_job_request,
            0,
            0,
        );
        let _ = txn_emitter
            .emit_txn_for(
                Duration::from_secs(emit_transactions_duration_secs),
                emit_job_request,
            )
            .await?;

        // Read the validator synced version
        let validator_synced_version = self.read_validator_synced_version();
        if validator_synced_version == 0.0 {
            return Err(anyhow::format_err!(
                "Validator synced zero transactions! Something has gone wrong!"
            ));
        }
        info!(
            "The validator is now synced at version: {}",
            validator_synced_version
        );

        // Restart the fullnode so that it starts state syncing to catch up
        info!(
            "Waiting for the fullnode to wake up: {}",
            self.fullnode_instance
        );
        self.fullnode_instance.start().await?;
        self.fullnode_instance
            .wait_server_ready(Instant::now() + Duration::from_secs(120))
            .await?;

        // Wait for the fullnode to catch up to the expected version
        info!(
            "The fullnode is now up. Waiting for it to state sync to the expected version: {}",
            validator_synced_version
        );
        let start_instant = Instant::now();
        while self.read_fullnode_synced_version() < validator_synced_version {
            time::sleep(Duration::from_secs(1)).await;
        }
        info!(
            "The fullnode has caught up to version: {}",
            validator_synced_version
        );

        // Calculate the state sync throughput
        let time_to_state_sync = start_instant.elapsed().as_secs();
        if time_to_state_sync == 0 {
            return Err(anyhow::format_err!(
                "The time taken to state sync was 0 seconds! Something has gone wrong!"
            ));
        }
        let state_sync_throughput = validator_synced_version as u64 / time_to_state_sync;
        let state_sync_throughput_message =
            format!("State sync throughput : {} txn/sec", state_sync_throughput,);
        info!("Time to state sync {:?}", time_to_state_sync);

        // Display the state sync throughput and report the results
        info!("{}", state_sync_throughput_message);
        context.report.report_text(state_sync_throughput_message);
        context
            .report
            .report_metric(self, "state_sync_throughput", state_sync_throughput as f64);

        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(EXPERIMENT_DURATION_TIMEOUT_SECS)
    }
}

impl StateSyncPerformance {
    fn read_fullnode_synced_version(&self) -> f64 {
        Self::read_synced_counter(&self.fullnode_instance)
    }

    fn read_validator_synced_version(&self) -> f64 {
        Self::read_synced_counter(&self.validator_instance)
    }

    // Reads the state sync "synced counter" for the given instance. If no
    // counter is found, returns zero.
    fn read_synced_counter(instance: &Instance) -> f64 {
        instance
            .counter(STATE_SYNC_COMMITTED_COUNTER_NAME)
            .unwrap_or(0.0)
    }
}

impl fmt::Display for StateSyncPerformance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StateSyncPerformance")
    }
}
