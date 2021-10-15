// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod compatibility_test;
pub mod partial_nodes_down_test;
pub mod performance_test;
pub mod twin_validator;

use diem_sdk::types::PeerId;
use forge::{EmitJobRequest, NetworkContext, NodeExt, Result, TxnEmitter, TxnStats, Version};
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

    ctx.swarm().health_check()?;
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
