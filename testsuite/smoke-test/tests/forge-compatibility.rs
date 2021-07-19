// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use diem_sdk::types::PeerId;
use forge::{
    forge_main, EmitJobRequest, ForgeConfig, InitialVersion, LocalFactory, NetworkContext,
    NetworkTest, NodeExt, Options, Result, SwarmExt, Test, TxnEmitter, Version,
};
use rand::SeedableRng;
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    ::diem_logger::Logger::init_for_testing();

    let tests = ForgeConfig::default()
        .with_initial_validator_count(NonZeroUsize::new(4).unwrap())
        .with_initial_version(InitialVersion::Oldest)
        .with_network_tests(&[&SimpleValidatorUpgrade]);

    let options = Options::from_args();
    forge_main(
        tests,
        LocalFactory::with_upstream_and_workspace()?,
        &options,
    )
}

fn batch_update<'t>(
    ctx: &mut NetworkContext<'t>,
    validators_to_update: &[PeerId],
    version: &Version,
) -> Result<()> {
    for validator in validators_to_update {
        ctx.swarm().upgrade_validator(*validator, version)?;
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    for validator in validators_to_update {
        ctx.swarm()
            .validator_mut(*validator)
            .unwrap()
            .wait_until_healthy(deadline)?;
    }

    Ok(())
}

fn generate_traffic<'t>(ctx: &mut NetworkContext<'t>, validators: &[PeerId]) -> Result<()> {
    let rt = Runtime::new()?;
    let duration = Duration::from_secs(10);
    let rng = SeedableRng::from_rng(ctx.core().rng())?;
    let validator_clients = ctx
        .swarm()
        .validators()
        .filter(|v| validators.contains(&v.peer_id()))
        .map(|n| n.async_json_rpc_client())
        .collect::<Vec<_>>();
    let mut emitter = TxnEmitter::new(ctx.swarm().chain_info(), rng);
    let _ =
        rt.block_on(emitter.emit_txn_for(duration, EmitJobRequest::default(validator_clients)))?;

    Ok(())
}

struct SimpleValidatorUpgrade;

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

        // Ensure that all validators are running the older version of the software
        let validators_to_downgrade = ctx
            .swarm()
            .validators()
            .filter(|v| v.version() != old_version)
            .map(|v| v.peer_id())
            .collect::<Vec<_>>();
        batch_update(ctx, &validators_to_downgrade, &old_version)?;

        // Generate some traffic
        generate_traffic(ctx, &all_validators)?;

        // Update the first Validator
        println!("upgrading first Validator");
        batch_update(ctx, &[first_node], &new_version)?;
        generate_traffic(ctx, &[first_node])?;

        // Update the rest of the first batch
        println!("upgrading rest of first batch");
        batch_update(ctx, &first_batch, &new_version)?;
        generate_traffic(ctx, &first_batch)?;

        ctx.swarm().fork_check()?;

        // Update the second batch
        println!("upgrading second batch");
        batch_update(ctx, &second_batch, &new_version)?;
        generate_traffic(ctx, &second_batch)?;

        println!("check swarm health");
        ctx.swarm().fork_check()?;

        Ok(())
    }
}
