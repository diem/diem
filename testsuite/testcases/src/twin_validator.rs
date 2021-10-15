// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::generate_traffic;
use forge::{NetworkContext, NetworkTest, Result, Test};
use tokio::time::Duration;
use rand::Rng;
use std::thread;
use diem_sdk::types::PeerId;

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
        let val = validators.remove(rnd.gen_range(0..validators.len()));
        let twin_id = val.peer_id();
        let twin_name = val.name().to_string();
        let validators = validators.iter().map(|v| {
            v.peer_id()
        }).collect::<Vec<_>>();
        make_twin_validator(ctx, twin_id)?;
        println!("Waiting for twin node to be up: {}", twin_name);
        ctx.swarm().health_check()?;
        thread::sleep(Duration::from_secs(20));

        // Generate some traffic
        let txn_stat = generate_traffic(ctx, &validators, duration)?;
        ctx.report
            .report_txn_stats(self.name().to_string(), txn_stat, duration);

        remove_twin_validator(ctx, twin_id)?;
        thread::sleep(Duration::from_secs(20));

        Ok(())
    }
}

fn make_twin_validator(ctx: &mut NetworkContext, id: PeerId) -> Result<()> {
    ctx.swarm().validator(id).unwrap().scale_replicas(2)
}

fn remove_twin_validator(ctx: &mut NetworkContext, id: PeerId) -> Result<()> {
    ctx.swarm().validator(id).unwrap().scale_replicas(1)
}
