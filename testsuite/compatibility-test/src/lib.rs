// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use diem_sdk::types::PeerId;
use forge::{
    EmitJobRequest, NetworkContext, NetworkTest, NodeExt, Result, SwarmExt, Test, TxnEmitter,
    TxnStats, Version,
};
use rand::SeedableRng;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

fn batch_update<'t>(
    ctx: &mut NetworkContext<'t>,
    validators_to_update: &[PeerId],
    version: &Version,
) -> Result<()> {
    for validator in validators_to_update {
        ctx.swarm().upgrade_validator(*validator, version)?;
    }

    let deadline = Instant::now() + Duration::from_secs(60);
    for validator in validators_to_update {
        ctx.swarm()
            .validator_mut(*validator)
            .unwrap()
            .wait_until_healthy(deadline)?;
    }

    Ok(())
}

pub fn generate_traffic<'t>(
    ctx: &mut NetworkContext<'t>,
    validators: &[PeerId],
    duration: Duration,
) -> Result<TxnStats> {
    let rt = Runtime::new()?;
    let rng = SeedableRng::from_rng(ctx.core().rng())?;
    let validator_clients = ctx
        .swarm()
        .validators()
        .filter(|v| validators.contains(&v.peer_id()))
        .map(|n| n.async_json_rpc_client())
        .collect::<Vec<_>>();
    let mut emitter = TxnEmitter::new(ctx.swarm().chain_info(), rng);
    let stats =
        rt.block_on(emitter.emit_txn_for(duration, EmitJobRequest::default(validator_clients)))?;

    Ok(stats)
}

pub struct SimpleValidatorUpgrade;

impl Test for SimpleValidatorUpgrade {
    fn name(&self) -> &'static str {
        "compatibility::simple-validator-upgrade"
    }
}

impl NetworkTest for SimpleValidatorUpgrade {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        // Get the different versions we're testing with
        let (old_version, new_version) = {
            let mut versions = ctx.swarm().versions().collect::<Vec<_>>();
            versions.sort();
            if versions.len() != 2 {
                bail!("exactly two different versions needed to run compat test");
            }

            (versions[0].clone(), versions[1].clone())
        };

        println!("testing upgrade from {} -> {}", old_version, new_version);

        // Split the swarm into 2 parts
        if ctx.swarm().validators().count() < 4 {
            bail!("compat test requires >= 4 validators");
        }
        let all_validators = ctx
            .swarm()
            .validators()
            .map(|v| v.peer_id())
            .collect::<Vec<_>>();
        let mut first_batch = all_validators.clone();
        let second_batch = first_batch.split_off(first_batch.len() / 2);
        let first_node = first_batch.pop().unwrap();
        let duration = Duration::from_secs(5);

        println!("1. Downgrade all validators to older version");
        // Ensure that all validators are running the older version of the software
        let validators_to_downgrade = ctx
            .swarm()
            .validators()
            .filter(|v| v.version() != old_version)
            .map(|v| v.peer_id())
            .collect::<Vec<_>>();
        batch_update(ctx, &validators_to_downgrade, &old_version)?;

        // Generate some traffic
        generate_traffic(ctx, &all_validators, duration)?;

        // Update the first Validator
        println!("2. upgrading first Validator");
        batch_update(ctx, &[first_node], &new_version)?;
        generate_traffic(ctx, &[first_node], duration)?;

        // Update the rest of the first batch
        println!("3. upgrading rest of first batch");
        batch_update(ctx, &first_batch, &new_version)?;
        generate_traffic(ctx, &first_batch, duration)?;

        ctx.swarm().fork_check()?;

        // Update the second batch
        println!("4. upgrading second batch");
        batch_update(ctx, &second_batch, &new_version)?;
        generate_traffic(ctx, &second_batch, duration)?;

        println!("5. check swarm health");
        ctx.swarm().fork_check()?;

        Ok(())
    }
}
