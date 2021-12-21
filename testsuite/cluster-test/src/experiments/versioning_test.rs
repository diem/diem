// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{
        compatibility_test::update_batch_instance, Context, Experiment, ExperimentParam,
    },
    instance,
    instance::Instance,
};
use anyhow::format_err;
use async_trait::async_trait;
use diem_logger::prelude::*;
use diem_sdk::{transaction_builder::TransactionFactory, types::LocalAccount};
use diem_transaction_builder::stdlib::encode_update_diem_version_script;
use diem_types::{chain_id::ChainId, transaction::TransactionPayload};
use forge::{execute_and_wait_transactions, TxnEmitter};
use language_e2e_tests::common_transactions::multi_agent_p2p_script_function;
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use std::{collections::HashSet, fmt, time::Duration};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ValidatorVersioningParams {
    #[structopt(
        long,
        default_value = "15",
        help = "Number of nodes to update in the first batch"
    )]
    pub count: usize,
    #[structopt(long, help = "Image tag of newer validator software")]
    pub updated_image_tag: String,
}

pub struct ValidatorVersioning {
    first_batch: Vec<Instance>,
    first_batch_lsr: Vec<Instance>,
    second_batch: Vec<Instance>,
    second_batch_lsr: Vec<Instance>,
    _full_nodes: Vec<Instance>,
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
        let (first_batch, second_batch) = cluster.split_n_validators_random(self.count);
        let first_batch = first_batch.into_validator_instances();
        let second_batch = second_batch.into_validator_instances();
        let mut first_batch_lsr = vec![];
        let mut second_batch_lsr = vec![];
        if !cluster.lsr_instances().is_empty() {
            first_batch_lsr = cluster.lsr_instances_for_validators(&first_batch);
            second_batch_lsr = cluster.lsr_instances_for_validators(&second_batch);
        }

        Self::E {
            first_batch,
            first_batch_lsr,
            second_batch,
            second_batch_lsr,
            _full_nodes: cluster.fullnode_instances().to_vec(),
            updated_image_tag: self.updated_image_tag,
        }
    }
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
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );

        // Mint a number of accounts
        txn_emitter
            .mint_accounts(
                &crate::util::emit_job_request_for_instances(
                    context.cluster.validator_instances().to_vec(),
                    context.global_emit_job_request,
                    0,
                    0,
                ),
                150,
            )
            .await?;
        let mut account = txn_emitter.take_account();
        let secondary_signer_account = txn_emitter.take_account();

        // Define the transaction generator
        //
        // TODO: In the future we may want to pass this functor as an argument to the experiment
        // to make versioning test extensible.
        // Define a multi-agent p2p transaction.
        let txn_payload = multi_agent_p2p_script_function(10);

        let tx_factory =
            TransactionFactory::new(ChainId::test()).with_transaction_expiration_time(420);
        let txn_gen = |account: &mut LocalAccount, secondary_signer_account: &LocalAccount| {
            account.sign_multi_agent_with_transaction_builder(
                vec![secondary_signer_account],
                tx_factory.payload(txn_payload.clone()),
            )
        };

        // grab a validator node
        let old_validator_node = context.cluster.random_validator_instance();
        let old_client = old_validator_node.rest_client();

        info!("1. Send a transaction using the new feature to a validator node");
        let txn1 = txn_gen(&mut account, &secondary_signer_account);
        if execute_and_wait_transactions(&old_client, &mut account, vec![txn1])
            .await
            .is_ok()
        {
            return Err(format_err!(
                "The transaction should be rejected as the new feature is not yet recognized \
                by any of the validator nodes"
            ));
        };
        info!("-- [Expected] The transaction is rejected by the validator node");

        info!("2. Update the first batch of validator nodes");
        update_batch_instance(
            context.cluster_swarm,
            &self.first_batch,
            &self.first_batch_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;

        // choose an updated validator
        let new_validator_node = self
            .first_batch
            .get(0)
            .expect("getting an updated validator instance requires a non-empty list");
        let new_client = new_validator_node.rest_client();

        info!("3. Send the transaction using the new feature to an updated validator node");
        let txn3 = txn_gen(&mut account, &secondary_signer_account);
        if execute_and_wait_transactions(&new_client, &mut account, vec![txn3])
            .await
            .is_ok()
        {
            return Err(format_err!(
                "The transaction should be rejected as the feature is under gating",
            ));
        }
        info!("-- The transaction is rejected as expected");

        info!("4. Update the rest of the validator nodes");
        update_batch_instance(
            context.cluster_swarm,
            &self.second_batch,
            &self.second_batch_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;

        info!("5. Send the transaction using the new feature to an updated validator node again");
        let txn4 = txn_gen(&mut account, &secondary_signer_account);
        if execute_and_wait_transactions(&new_client, &mut account, vec![txn4])
            .await
            .is_ok()
        {
            return Err(format_err!(
                "The transaction should be rejected as the feature is still gated",
            ));
        }
        info!("-- The transaction is still rejected as expected, because the new feature is gated");

        info!("6. Activate the new feature multi agent");
        let mut diem_root_account = &mut context.root_account;
        let allowed_nonce = 0;
        let update_txn = diem_root_account.sign_with_transaction_builder(tx_factory.payload(
            TransactionPayload::Script(encode_update_diem_version_script(allowed_nonce, 3)),
        ));
        execute_and_wait_transactions(&new_client, &mut diem_root_account, vec![update_txn])
            .await?;

        info!("7. Send the transaction using the new feature after Diem version update");
        let txn5 = txn_gen(&mut account, &secondary_signer_account);
        execute_and_wait_transactions(&new_client, &mut account, vec![txn5]).await?;
        info!("-- [Expected] The transaction goes through");

        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(15 * 60)
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
