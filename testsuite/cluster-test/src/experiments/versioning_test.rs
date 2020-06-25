// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{
    collections::HashSet,
    fmt,
    time::{Duration, Instant},
};

use rand::Rng;

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
    tx_emitter::{execute_and_wait_transactions, EmitJobRequest},
};
use async_trait::async_trait;
use futures::future::try_join_all;
use libra_logger::prelude::*;
use libra_types::{
    account_config::{lbr_type_tag, LBR_NAME},
    on_chain_config::LibraVersion,
    transaction::{helpers::create_user_txn, TransactionPayload},
};
use structopt::StructOpt;
use transaction_builder::{
    encode_transfer_with_metadata_script, encode_update_libra_version_script,
};

#[derive(StructOpt, Debug)]
pub struct ValidatorVersioningParams {
    #[structopt(long, default_value = "10", help = "Number of nodes to reboot")]
    pub count: usize,
    #[structopt(long, help = "Image tag of newer validator software")]
    pub updated_image_tag: String,
}

pub struct ValidatorVersioning {
    first_batch: Vec<Instance>,
    second_batch: Vec<Instance>,
    full_nodes: Vec<Instance>,
    updated_image_tag: String,
}

impl ExperimentParam for ValidatorVersioningParams {
    type E = ValidatorVersioning;
    fn build(self, cluster: &Cluster) -> Self::E {
        if self.count > cluster.validator_instances().len() {
            panic!(
                "Can not reboot {} validators in cluster with {} instances",
                self.count,
                cluster.validator_instances().len()
            );
        }
        let mut instances = Vec::with_capacity(self.count);
        let mut all_instances = cluster.validator_instances().to_vec();
        let mut rnd = rand::thread_rng();
        for _i in 0..self.count {
            let instance = all_instances.remove(rnd.gen_range(0, all_instances.len()));
            instances.push(instance);
        }

        Self::E {
            first_batch: instances,
            second_batch: all_instances,
            full_nodes: cluster.fullnode_instances().to_vec(),
            updated_image_tag: self.updated_image_tag,
        }
    }
}

async fn update_batch_instance(
    context: &mut Context<'_>,
    updated_instance: &[Instance],
    updated_tag: String,
) -> anyhow::Result<()> {
    let deadline = Instant::now() + Duration::new(2 * 60, 0);

    // 1. Stop Existing instances.
    let futures: Vec<_> = updated_instance.iter().map(Instance::stop).collect();
    try_join_all(futures).await?;

    // 2. Reinstantiate a set of new nodes.
    let futures: Vec<_> = updated_instance
        .iter()
        .map(|instance| {
            let mut newer_config = instance.instance_config().clone();
            newer_config.replace_tag(updated_tag.clone()).unwrap();
            context
                .cluster_swarm
                .spawn_new_instance(newer_config, false)
        })
        .collect();
    let instances = try_join_all(futures).await?;

    // 3. Wait for the instances to recover.
    let futures: Vec<_> = instances
        .iter()
        .map(|instance| instance.wait_json_rpc(deadline))
        .collect();
    try_join_all(futures).await?;
    Ok(())
}

#[async_trait]
impl Experiment for ValidatorVersioning {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.first_batch)
            .union(&instance::instancelist_to_set(&self.second_batch))
            .cloned()
            .collect()
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        // Mint a number of accounts
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

        info!("1. Changing the images for the instances in the first batch");
        update_batch_instance(context, &self.first_batch, self.updated_image_tag.clone()).await?;

        info!("2. Send a transaction to make sure it is not rejected nor cause any fork");
        let full_node = context.cluster.random_full_node_instance();
        let mut full_node_client = full_node.json_rpc_client();
        let mut account_1 = context.tx_emitter.take_account();
        let account_2 = context.tx_emitter.take_account();

        let txn_payload = TransactionPayload::Script(encode_transfer_with_metadata_script(
            lbr_type_tag(),
            account_2.address,
            1,
            vec![],
            vec![],
        ));

        let txn1 = create_user_txn(
            &account_1.key_pair,
            txn_payload.clone(),
            account_1.address,
            account_1.sequence_number,
            123456,
            0,
            LBR_NAME.to_owned(),
            10,
        )
        .expect("Failed to create signed transaction");
        account_1.sequence_number += 1;

        execute_and_wait_transactions(&mut full_node_client, &mut account_1, vec![txn1.clone()])
            .await?;

        info!("3. Change the rest of the images in the second batch");
        update_batch_instance(context, &self.second_batch, self.updated_image_tag.clone()).await?;

        info!("4. Send a transaction to make sure this feature is still not activated.");
        let txn2 = create_user_txn(
            &account_1.key_pair,
            txn_payload.clone(),
            account_1.address,
            account_1.sequence_number,
            123456,
            0,
            LBR_NAME.to_owned(),
            10,
        )
        .expect("Failed to create signed transaction");
        account_1.sequence_number += 1;
        execute_and_wait_transactions(&mut full_node_client, &mut account_1, vec![txn2.clone()])
            .await?;

        info!("5. Send a transaction to activate such feature");
        let mut faucet_account = context.tx_emitter.load_faucet_account(&full_node).await?;
        let update_txn = create_user_txn(
            &faucet_account.key_pair,
            TransactionPayload::Script(encode_update_libra_version_script(LibraVersion {
                major: 11,
            })),
            faucet_account.address,
            faucet_account.sequence_number,
            123456,
            0,
            LBR_NAME.to_owned(),
            10,
        )
        .expect("Failed to create signed transaction");
        faucet_account.sequence_number += 1;

        execute_and_wait_transactions(
            &mut full_node_client,
            &mut faucet_account,
            vec![update_txn.clone()],
        )
        .await?;

        info!("6. Send a transaction to make sure it passes the full node mempool but will not be committed by updated validators.");
        let txn3 = create_user_txn(
            &account_1.key_pair,
            txn_payload,
            account_1.address,
            account_1.sequence_number,
            123456,
            0,
            LBR_NAME.to_owned(),
            10,
        )
        .expect("Failed to create signed transaction");

        full_node_client
            .submit_transaction(txn3.clone())
            .await
            .expect("Transaction should pass the full node mempool");

        execute_and_wait_transactions(&mut full_node_client, &mut account_1, vec![txn3.clone()])
            .await
            .unwrap_err();

        info!("7. Change the images for the full nodes");
        update_batch_instance(context, &self.full_nodes, self.updated_image_tag.clone()).await?;

        info!("8. Send a transaction to make sure it gets dropped by the full node mempool.");

        let updated_full_node = context
            .cluster
            .random_full_node_instance()
            .json_rpc_client();
        updated_full_node
            .submit_transaction(txn3)
            .await
            .unwrap_err();
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(20 * 60)
    }
}

impl fmt::Display for ValidatorVersioning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Updating [")?;
        for instance in self.first_batch.iter() {
            write!(f, "{}, ", instance)?;
        }
        for instance in self.second_batch.iter() {
            write!(f, "{}, ", instance)?;
        }
        write!(f, "]")?;
        writeln!(f, "Updated Config: {:?}", self.updated_image_tag)
    }
}
