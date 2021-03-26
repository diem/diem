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
    tx_emitter::{execute_and_wait_transactions, AccountData, EmitJobRequest},
};
use anyhow::format_err;
use async_trait::async_trait;
use diem_logger::prelude::*;
use diem_transaction_builder::stdlib::encode_update_diem_version_script;
use diem_types::{
    account_config::XUS_NAME,
    chain_id::ChainId,
    transaction::{helpers::create_user_txn, ScriptFunction, TransactionPayload},
};
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
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
        // Mint a number of accounts
        context
            .tx_emitter
            .mint_accounts(
                &EmitJobRequest::for_instances(
                    context.cluster.validator_instances().to_vec(),
                    context.global_emit_job_request,
                    0,
                    0,
                ),
                150,
            )
            .await?;
        let mut account = context.tx_emitter.take_account();

        // Define the transaction generator
        //
        // TODO: In the future we may want to pass this functor as an argument to the experiment
        // to make versioning test extensible.
        let txn_payload = TransactionPayload::ScriptFunction(ScriptFunction::new(
            ModuleId::new(account.address, Identifier::new("PHANTOM_MODULE").unwrap()),
            Identifier::new("PHANTOM_FUNCTION").unwrap(),
            vec![],
            vec![],
        ));

        let txn_gen = |account: &AccountData| {
            create_user_txn(
                &account.key_pair,
                txn_payload.clone(),
                account.address,
                account.sequence_number,
                123456,
                0,
                XUS_NAME.to_owned(),
                10,
                ChainId::test(),
            )
            .map_err(|e| format_err!("Failed to create signed transaction: {}", e))
        };

        // grab a validator node
        let old_validator_node = context.cluster.random_validator_instance();
        let mut old_client = old_validator_node.json_rpc_client();

        info!("1. Send a transaction using the new feature to a validator node");
        let txn1 = txn_gen(&account)?;
        if execute_and_wait_transactions(&mut old_client, &mut account, vec![txn1])
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
            context,
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
        let mut new_client = new_validator_node.json_rpc_client();

        info!("3. Send the transaction using the new feature to an updated validator node");
        let txn3 = txn_gen(&account)?;
        if execute_and_wait_transactions(&mut new_client, &mut account, vec![txn3])
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
            context,
            &self.second_batch,
            &self.second_batch_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;

        info!("5. Send the transaction using the new feature to an updated validator node again");
        let txn4 = txn_gen(&account)?;
        if execute_and_wait_transactions(&mut new_client, &mut account, vec![txn4])
            .await
            .is_ok()
        {
            return Err(format_err!(
                "The transaction should be rejected as the feature is still gated",
            ));
        }
        info!("-- The transaction is still rejected as expected, because the new feature is gated");

        info!("6. Activate the new feature");
        let mut diem_root_account = context
            .tx_emitter
            .load_diem_root_account(&new_client)
            .await?;
        let allowed_nonce = 0;
        let update_txn = create_user_txn(
            &diem_root_account.key_pair,
            TransactionPayload::Script(encode_update_diem_version_script(allowed_nonce, 11)),
            diem_root_account.address,
            diem_root_account.sequence_number,
            123456,
            0,
            XUS_NAME.to_owned(),
            10,
            ChainId::test(),
        )
        .map_err(|e| format_err!("Failed to create signed transaction: {}", e))?;
        diem_root_account.sequence_number += 1;
        execute_and_wait_transactions(&mut new_client, &mut diem_root_account, vec![update_txn])
            .await?;

        info!("7. Send the transaction using the new feature after Diem version update");
        let txn5 = txn_gen(&account)?;
        account.sequence_number += 1;
        execute_and_wait_transactions(&mut new_client, &mut account, vec![txn5]).await?;
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
