// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::generate_traffic;
use forge::{NetworkContext, NetworkTest, Result, Test, Validator};
use tokio::time::Duration;
use rand::Rng;
use std::thread;

pub struct TwinValidator;

impl Test for TwinValidator {
    fn name(&self) -> &'static str {
        "twin-validator"
    }
}

impl NetworkTest for TwinValidator {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        let duration = Duration::from_secs(240);
        let all_validators = ctx
            .swarm()
            .validators()
            .collect::<Vec<_>>();
        let mut validators = all_validators.clone();
        let mut rnd = rand::thread_rng();
        let twin_node = validators.remove(rnd.gen_range(0..validators.len()));
        let validators = validators.iter().map(|v| {
            v.peer_id()
        }).collect::<Vec<_>>();
        make_twin_validator(twin_node)?;
        println!("Waiting for twin node to be up: {}", twin_node.name());
        ctx.swarm().health_check()?;
        thread::sleep(Duration::from_secs(20));

        // Generate some traffic
        let txn_stat = generate_traffic(ctx, &validators, duration)?;
        ctx.report
            .report_txn_stats(self.name().to_string(), txn_stat, duration);

        remove_twin_validator(twin_node)?;
        thread::sleep(Duration::from_secs(20));

        Ok(())
    }
}

fn make_twin_validator(node: &dyn Validator) -> Result<()> {
    node.scale_replicas(2)
}

fn remove_twin_validator(node: &dyn Validator) -> Result<()> {
    node.scale_replicas(1)
}