// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_sdk::{
    client::Client,
    transaction_builder::TransactionFactory,
    types::{
        account_address::AccountAddress, account_config::treasury_compliance_account_address,
        chain_id::ChainId, diem_id_identifier::DiemIdVaspDomainIdentifier,
        transaction::SignedTransaction, LocalAccount,
    },
};
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
pub struct DiemIdDomainParams {
    domain: DiemIdVaspDomainIdentifier,
    address: AccountAddress,
}

impl DiemIdDomainParams {
    pub fn domain(&self) -> &DiemIdVaspDomainIdentifier {
        &self.domain
    }

    pub fn address(&self) -> &AccountAddress {
        &self.address
    }
}

pub struct DiemIdDomainService {
    transaction_factory: TransactionFactory,
    treasury_account: Mutex<LocalAccount>,
    client: Client,
}

impl DiemIdDomainService {
    pub fn new(server_url: String, chain_id: ChainId, private_key_file: String) -> Self {
        let treasury_account = LocalAccount::new(
            treasury_compliance_account_address(),
            generate_key::load_key(&private_key_file),
            0,
        );
        let client = Client::new(server_url);
        DiemIdDomainService {
            treasury_account: Mutex::new(treasury_account),
            transaction_factory: TransactionFactory::new(chain_id)
                .with_transaction_expiration_time(30),
            client,
        }
    }

    pub async fn add_diem_id_domain(
        &self,
        params: DiemIdDomainParams,
    ) -> Result<SignedTransaction> {
        let tc_seq = self.get_tc_sequence_number().await?;
        let mut treasury_account = self.treasury_account.lock().unwrap();
        if tc_seq > treasury_account.sequence_number() {
            *treasury_account.sequence_number_mut() = tc_seq;
        }
        let builder = self.transaction_factory.add_diem_id_domain(
            *params.address(),
            params.domain().as_str().as_bytes().to_vec(),
        );
        let txn = treasury_account.sign_with_transaction_builder(builder);

        let response = self.client.submit(&txn).await;

        // If there was an issue submitting a transaction we should just reset our sequence_numbers
        // to what was on chain
        if response.is_err() {
            *self.treasury_account.lock().unwrap().sequence_number_mut() = tc_seq;
        }

        Ok(txn)
    }

    async fn get_tc_sequence_number(&self) -> Result<u64> {
        let response = self
            .client
            .get_account(treasury_compliance_account_address())
            .await?
            .into_inner();

        Ok(response
            .ok_or_else(|| anyhow::format_err!("treasury compliance account not found"))?
            .sequence_number)
    }
}
