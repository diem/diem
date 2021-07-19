// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_sdk::client::BlockingClient;
use forge::{
    forge_main, EmitJobRequest, ForgeConfig, InitialVersion, LocalFactory, NetworkContext,
    NetworkTest, NodeExt, Options, Result, Test, TxnEmitter,
};
use rand::SeedableRng;
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    let tests = ForgeConfig::default()
        .with_initial_validator_count(NonZeroUsize::new(4).unwrap())
        .with_initial_version(InitialVersion::Oldest)
        .with_network_tests(&[&SimpleValidatorUpgrade, &VersionTest]);

    let options = Options::from_args();
    forge_main(
        tests,
        LocalFactory::with_upstream_and_workspace()?,
        &options,
    )
}

struct SimpleValidatorUpgrade;

impl Test for SimpleValidatorUpgrade {
    fn name(&self) -> &'static str {
        "compatibility::simple-validator-upgrade"
    }
}

impl NetworkTest for SimpleValidatorUpgrade {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        let swarm = ctx.swarm();

        let mut versions = swarm.versions().collect::<Vec<_>>();
        versions.sort();
        let old = versions[0].clone();
        let _new = versions[1].clone();

        let peer_id = swarm.validators().next().unwrap().peer_id();

        // let before_rev = debug_client.get_revision()?;
        let before_rev = swarm.validator(peer_id).unwrap().version();
        println!("before rev = {}", before_rev);

        println!("Upgrading Validator");
        swarm.upgrade_validator(peer_id, &old)?;

        let deadline = Instant::now() + Duration::from_secs(5);
        swarm
            .validator_mut(peer_id)
            .unwrap()
            .wait_until_healthy(deadline)?;
        println!("Upgrade complete");

        // let after_rev = debug_client.get_revision()?;
        let after_rev = swarm.validator(peer_id).unwrap().version();
        println!("after rev = {}", after_rev);

        Ok(())
    }
}

struct VersionTest;

impl Test for VersionTest {
    fn name(&self) -> &'static str {
        "compatibility::version-test"
    }
}

impl NetworkTest for VersionTest {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        {
            let swarm = ctx.swarm();

            let node = swarm.validators().next().unwrap();
            let last_committed_round_str = "diem_consensus_last_committed_round{}";
            let last_committed_version_str = "diem_consensus_last_committed_version{}";
            let storage_version_str = "diem_storage_latest_transaction_version{}";
            let last_round = node.get_metric(last_committed_round_str)?.unwrap();
            println!("last_round: {}", last_round);

            let last_version = node.get_metric(last_committed_version_str)?.unwrap();
            println!("last_version: {}", last_version);

            let storage_version = node.get_metric(storage_version_str)?.unwrap();
            println!("storage_version: {}", storage_version);

            let client = BlockingClient::new(node.json_rpc_endpoint());
            let metadata = client.get_metadata()?.into_inner();
            println!("metadata: {:#?}", metadata);
            let metadata = client
                .get_metadata_by_version((last_version - 1) as u64)?
                .into_inner();
            println!("metadata: {:#?}", metadata);
        }

        let duration = Duration::from_secs(10);
        let rng = SeedableRng::from_rng(ctx.core().rng()).unwrap();
        let validator_clients = ctx
            .swarm()
            .validators()
            .into_iter()
            .map(|n| n.async_json_rpc_client())
            .collect::<Vec<_>>();
        let mut emitter = TxnEmitter::new(ctx.swarm().chain_info(), rng);
        let rt = Runtime::new().unwrap();
        let stats = rt
            .block_on(emitter.emit_txn_for(duration, EmitJobRequest::default(validator_clients)))
            .unwrap();
        ctx.report
            .report_txn_stats(self.name().to_string(), stats, duration);
        ctx.report.print_report();

        {
            let swarm = ctx.swarm();

            let node = swarm.validators().next().unwrap();
            let last_committed_round_str = "diem_consensus_last_committed_round{}";
            let last_committed_version_str = "diem_consensus_last_committed_version{}";
            let storage_version_str = "diem_storage_latest_transaction_version{}";
            let last_round = node.get_metric(last_committed_round_str)?.unwrap();
            println!("last_round: {}", last_round);

            let last_version = node.get_metric(last_committed_version_str)?.unwrap();
            println!("last_version: {}", last_version);

            let storage_version = node.get_metric(storage_version_str)?.unwrap();
            println!("storage_version: {}", storage_version);

            let client = BlockingClient::new(node.json_rpc_endpoint());
            let metadata = client.get_metadata()?.into_inner();
            println!("metadata: {:#?}", metadata);
            let metadata = client
                .get_metadata_by_version((last_version - 1) as u64)?
                .into_inner();
            println!("metadata: {:#?}", metadata);
        }

        Ok(())
    }
}
