// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance::Instance,
};
use anyhow::ensure;
use async_trait::async_trait;
use diem_client::Client;
use diem_logger::prelude::*;
use diem_operational_tool::json_rpc::JsonRpcClientWrapper;
use diem_sdk::transaction_builder::TransactionFactory;
use diem_types::{
    account_address::AccountAddress,
    chain_id::ChainId,
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config::{ConsensusConfigV2, OnChainConsensusConfig},
};
use forge::{execute_and_wait_transactions, TxnEmitter};
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use std::{
    collections::HashSet,
    fmt,
    time::{Duration, Instant},
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ReconfigurationParams {
    #[structopt(long, default_value = "101", help = "Number of epochs to trigger")]
    pub count: u64,
    #[structopt(long, help = "Emit p2p transfer transactions during experiment")]
    pub emit_txn: bool,
}

pub struct Reconfiguration {
    affected_peer_id: AccountAddress,
    affected_pod_name: String,
    count: u64,
    emit_txn: bool,
}

impl ExperimentParam for ReconfigurationParams {
    type E = Reconfiguration;
    fn build(self, cluster: &Cluster) -> Self::E {
        let full_node = cluster.random_fullnode_instance();
        let client = JsonRpcClientWrapper::new(full_node.json_rpc_url().into());
        let validator_info = client
            .validator_set(None)
            .expect("Unable to fetch validator set");
        let affected_peer_id = *validator_info[0].account_address();
        let validator_config = client
            .validator_config(affected_peer_id)
            .expect("Unable to fetch validator config");
        let affected_pod_name = std::str::from_utf8(&validator_config.human_name)
            .unwrap()
            .to_string();
        Self::E {
            affected_peer_id,
            affected_pod_name,
            count: self.count,
            emit_txn: self.emit_txn,
        }
    }
}

async fn expect_epoch(
    client: &Client,
    known_version: u64,
    expected_epoch: u64,
) -> anyhow::Result<u64> {
    let state_proof = client.get_state_proof(known_version).await?.into_inner();
    let li: LedgerInfoWithSignatures = bcs::from_bytes(&state_proof.ledger_info_with_signatures)?;
    let epoch = li.ledger_info().next_block_epoch();
    ensure!(
        epoch == expected_epoch,
        "Expect epoch {}, actual {}",
        expected_epoch,
        epoch
    );
    info!("Epoch {} is committed", epoch);
    Ok(li.ledger_info().version())
}

#[async_trait]
impl Experiment for Reconfiguration {
    fn affected_validators(&self) -> HashSet<String> {
        let mut nodes = HashSet::new();
        nodes.insert(self.affected_pod_name.clone());
        nodes
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );
        let full_node = context.cluster.random_fullnode_instance();
        let tx_factory = TransactionFactory::new(ChainId::test());
        let full_node_client = full_node.rest_client();
        let full_node_jsonrpc_client = full_node.json_rpc_client();
        let mut diem_root_account = &mut context.root_account;
        let allowed_nonce = 0;
        let emit_job = if self.emit_txn {
            info!("Start emitting txn");
            let instances: Vec<Instance> = context
                .cluster
                .validator_instances()
                .iter()
                .filter(|i| *i.peer_name() != self.affected_pod_name)
                .cloned()
                .collect();
            Some(
                txn_emitter
                    .start_job(crate::util::emit_job_request_for_instances(
                        instances,
                        context.global_emit_job_request,
                        0,
                        0,
                    ))
                    .await?,
            )
        } else {
            None
        };

        let timer = Instant::now();
        let mut version = expect_epoch(&full_node_jsonrpc_client, 0, 1).await?;
        {
            info!("Remove and add back {}.", self.affected_pod_name);
            let validator_name = self.affected_pod_name.as_bytes().to_vec();
            let remove_txn = diem_root_account.sign_with_transaction_builder(
                tx_factory.remove_validator_and_reconfigure(
                    allowed_nonce,
                    validator_name.clone(),
                    self.affected_peer_id,
                ),
            );
            execute_and_wait_transactions(
                &full_node_client,
                &mut diem_root_account,
                vec![remove_txn],
            )
            .await?;
            version = expect_epoch(&full_node_jsonrpc_client, version, 2).await?;
            let add_txn = diem_root_account.sign_with_transaction_builder(
                tx_factory.add_validator_and_reconfigure(
                    allowed_nonce,
                    validator_name.clone(),
                    self.affected_peer_id,
                ),
            );
            execute_and_wait_transactions(&full_node_client, &mut diem_root_account, vec![add_txn])
                .await?;
            version = expect_epoch(&full_node_jsonrpc_client, version, 3).await?;
        }

        {
            info!("Switch decoupled-execution on and off repetitively.");
            let upgrade_config = OnChainConsensusConfig::V2(ConsensusConfigV2 {
                two_chain: true,
                decoupled_execution: true,
                back_pressure_limit: 10,
                exclude_round: 20,
            });
            let downgrade_config = OnChainConsensusConfig::default();
            for i in 1..self.count / 2 {
                let upgrade_txn = diem_root_account.sign_with_transaction_builder(
                    tx_factory.update_diem_consensus_config(
                        allowed_nonce,
                        bcs::to_bytes(&upgrade_config).unwrap(),
                    ),
                );
                execute_and_wait_transactions(
                    &full_node_client,
                    &mut diem_root_account,
                    vec![upgrade_txn],
                )
                .await?;
                version = expect_epoch(&full_node_jsonrpc_client, version, (i + 1) * 2).await?;
                let downgrade_txn = diem_root_account.sign_with_transaction_builder(
                    tx_factory.update_diem_consensus_config(
                        allowed_nonce,
                        bcs::to_bytes(&downgrade_config).unwrap(),
                    ),
                );
                execute_and_wait_transactions(
                    &full_node_client,
                    &mut diem_root_account,
                    vec![downgrade_txn],
                )
                .await?;
                version = expect_epoch(&full_node_jsonrpc_client, version, (i + 1) * 2 + 1).await?;
            }
        }

        if self.count % 2 == 1 {
            let magic_number = 42;
            info!("Bump DiemVersion to {}", magic_number);
            let update_txn = diem_root_account.sign_with_transaction_builder(
                TransactionFactory::new(ChainId::test())
                    .update_diem_version(allowed_nonce, magic_number),
            );
            execute_and_wait_transactions(
                &full_node_client,
                &mut diem_root_account,
                vec![update_txn],
            )
            .await?;
            expect_epoch(&full_node_jsonrpc_client, version, self.count + 1).await?;
        }
        let elapsed = timer.elapsed();
        if let Some(job) = emit_job {
            let stats = txn_emitter.stop_job(job).await;
            context
                .report
                .report_txn_stats(self.to_string(), stats, elapsed, "");
        } else {
            context.report.report_text(format!(
                "{} finished in {} seconds",
                self.to_string(),
                elapsed.as_secs()
            ));
        }

        Ok(())
    }

    fn deadline(&self) -> Duration {
        // allow each epoch to take 20 secs
        Duration::from_secs(self.count as u64 * 10)
    }
}

impl fmt::Display for Reconfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reconfiguration: total epoch: {}", self.count)
    }
}
