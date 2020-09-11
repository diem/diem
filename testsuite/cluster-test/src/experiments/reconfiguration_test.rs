// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    tx_emitter::{execute_and_wait_transactions, gen_submit_transaction_request, EmitJobRequest},
};
use async_trait::async_trait;
use libra_logger::prelude::*;
use libra_operational_tool::json_rpc::JsonRpcClientWrapper;
use libra_types::{account_address::AccountAddress, chain_id::ChainId};
use std::{collections::HashSet, fmt, time::Duration};
use structopt::StructOpt;
use transaction_builder::{
    encode_add_validator_and_reconfigure_script, encode_remove_validator_and_reconfigure_script,
    encode_update_libra_version_script,
};

#[derive(StructOpt, Debug)]
pub struct ReconfigurationParams {
    #[structopt(long, default_value = "101", help = "Number of epochs to trigger")]
    pub count: usize,
}

pub struct Reconfiguration {
    affected_peer_id: AccountAddress,
    affected_pod_name: String,
    count: usize,
}

impl ExperimentParam for ReconfigurationParams {
    type E = Reconfiguration;
    fn build(self, cluster: &Cluster) -> Self::E {
        let full_node = cluster.random_fullnode_instance();
        let client = JsonRpcClientWrapper::new(full_node.json_rpc_url().into_string());
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
        }
    }
}

#[async_trait]
impl Experiment for Reconfiguration {
    fn affected_validators(&self) -> HashSet<String> {
        let mut nodes = HashSet::new();
        nodes.insert(self.affected_pod_name.clone());
        nodes
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        info!("Start Minting first");
        context
            .tx_emitter
            .mint_accounts(
                &EmitJobRequest::for_instances(
                    context.cluster.validator_instances().to_vec(),
                    context.global_emit_job_request,
                ),
                150,
            )
            .await?;

        let full_node = context.cluster.random_fullnode_instance();
        let mut full_node_client = full_node.json_rpc_client();
        let mut libra_root_account = context
            .tx_emitter
            .load_libra_root_account(&full_node)
            .await?;
        let allowed_nonce = 0;

        info!(
            "Remove and add back {} repetitively",
            self.affected_pod_name
        );
        let validator_name = self.affected_pod_name.as_bytes().to_vec();
        for i in 0..self.count / 2 {
            let remove_txn = gen_submit_transaction_request(
                encode_remove_validator_and_reconfigure_script(
                    allowed_nonce,
                    validator_name.clone(),
                    self.affected_peer_id,
                ),
                &mut libra_root_account,
                true,
                ChainId::test(),
            );
            execute_and_wait_transactions(
                &mut full_node_client,
                &mut libra_root_account,
                vec![remove_txn],
            )
            .await?;
            info!("Epoch {} committed", (i + 1) * 2);
            let add_txn = gen_submit_transaction_request(
                encode_add_validator_and_reconfigure_script(
                    allowed_nonce,
                    validator_name.clone(),
                    self.affected_peer_id,
                ),
                &mut libra_root_account,
                true,
                ChainId::test(),
            );
            execute_and_wait_transactions(
                &mut full_node_client,
                &mut libra_root_account,
                vec![add_txn],
            )
            .await?;
            info!("Epoch {} committed", (i + 1) * 2 + 1);
        }

        if self.count % 2 == 1 {
            let magic_number = 42;
            info!("Bump LibraVersion to {}", magic_number);
            let update_txn = gen_submit_transaction_request(
                encode_update_libra_version_script(allowed_nonce, magic_number),
                &mut libra_root_account,
                true,
                ChainId::test(),
            );

            execute_and_wait_transactions(
                &mut full_node_client,
                &mut libra_root_account,
                vec![update_txn],
            )
            .await?;
            info!("Epoch {} committed", self.count + 1);
        }

        Ok(())
    }

    fn deadline(&self) -> Duration {
        // allow each epoch to take 20 secs
        Duration::from_secs(self.count as u64 * 20)
    }
}

impl fmt::Display for Reconfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Affected peer: {}, pod_name: {}, total epoch: {}",
            self.affected_peer_id, self.affected_pod_name, self.count
        )
    }
}
