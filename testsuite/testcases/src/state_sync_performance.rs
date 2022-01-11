// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::generate_traffic;
use forge::{NetworkContext, NetworkTest, Result, Test};
use rand::{
    rngs::{OsRng, StdRng},
    seq::IteratorRandom,
    Rng, SeedableRng,
};
use std::{thread, time::Instant};
use tokio::{runtime::Runtime, time::Duration};

const STATE_SYNC_COMMITTED_COUNTER_NAME: &str = "diem_state_sync_version.synced";

pub struct StateSyncPerformance;

impl Test for StateSyncPerformance {
    fn name(&self) -> &'static str {
        "StateSyncPerformance"
    }
}

impl NetworkTest for StateSyncPerformance {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        let mut rng = StdRng::from_seed(OsRng.gen());
        let duration = Duration::from_secs(60);
        let all_validators = ctx
            .swarm()
            .validators()
            .map(|v| v.peer_id())
            .collect::<Vec<_>>();
        let all_fullnodes = ctx
            .swarm()
            .full_nodes()
            .map(|v| v.peer_id())
            .collect::<Vec<_>>();

        // 1. pick one fullnode to stop
        let fullnode_id = all_fullnodes.iter().choose(&mut rng).unwrap();
        ctx.swarm().full_node_mut(*fullnode_id).unwrap().stop()?;

        // 2. emit txn to validators
        generate_traffic(ctx, &all_validators, duration, 0, None)?;

        // 3. read the validator synced version
        let validator_id = all_validators.iter().choose(&mut rng).unwrap();
        let validator = ctx.swarm().validator(*validator_id).unwrap();
        let validator_metric_port = validator.expose_metric()?;
        let validator_synced_version = validator
            .counter(STATE_SYNC_COMMITTED_COUNTER_NAME, validator_metric_port)
            .unwrap_or(0.0);
        if validator_synced_version == 0.0 {
            return Err(anyhow::format_err!(
                "Validator synced zero transactions! Something has gone wrong!"
            ));
        }
        println!(
            "The validator is now synced at version: {}",
            validator_synced_version
        );

        // 4. restart the fullnode so that it starts state syncing to catch up
        let fullnode = ctx.swarm().full_node_mut(*fullnode_id).unwrap();
        // do data cleanup
        fullnode.clear_storage()?;
        println!("The fullnode is going to restart");
        let runtime = Runtime::new().unwrap();
        runtime.block_on(fullnode.start())?;
        println!(
            "The fullnode is now up. Waiting for it to state sync to the expected version: {}",
            validator_synced_version
        );
        let start_instant = Instant::now();
        let fullnode_metric_port = fullnode.expose_metric()?;
        while fullnode
            .counter(STATE_SYNC_COMMITTED_COUNTER_NAME, fullnode_metric_port)
            .unwrap_or(0.0)
            < validator_synced_version
        {
            thread::sleep(Duration::from_secs(1));
        }
        println!(
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
        println!("Time to state sync: {:?} secs", time_to_state_sync);
        // Display the state sync throughput and report the results
        println!("{}", state_sync_throughput_message);
        ctx.report.report_text(state_sync_throughput_message);
        ctx.report.report_metric(
            self.name(),
            "state_sync_throughput",
            state_sync_throughput as f64,
        );

        Ok(())
    }
}
