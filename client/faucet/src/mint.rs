// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_crypto::traits::SigningKey;
use libra_types::account_config::{
    testnet_dd_account_address, treasury_compliance_account_address, type_tag_for_currency_code,
    LBR_NAME,
};
use std::{convert::From, fmt};

#[derive(Debug)]
pub enum Response {
    DDAccountNextSeqNum(u64),
    SubmittedTxns(Vec<libra_types::transaction::SignedTransaction>),
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::DDAccountNextSeqNum(v1) => write!(f, "{}", v1),
            Response::SubmittedTxns(v2) => {
                write!(f, "{}", hex::encode(lcs::to_bytes(&v2).unwrap()))
            }
        }
    }
}

#[derive(serde_derive::Deserialize)]
pub struct MintParams {
    pub amount: u64,
    pub currency_code: move_core_types::identifier::Identifier,
    pub auth_key: libra_types::transaction::authenticator::AuthenticationKey,
    pub return_txns: Option<bool>,
}

impl MintParams {
    fn currency_code(&self) -> move_core_types::language_storage::TypeTag {
        type_tag_for_currency_code(self.currency_code.to_owned())
    }

    fn create_parent_vasp_account_script(
        &self,
        seq: u64,
    ) -> libra_types::transaction::TransactionPayload {
        let receiver = self.auth_key.derived_address();
        libra_types::transaction::TransactionPayload::Script(
            transaction_builder_generated::stdlib::encode_create_parent_vasp_account_script(
                self.currency_code(),
                0, // sliding nonce
                receiver,
                self.auth_key.prefix().to_vec(),
                format!("No. {}", seq).as_bytes().to_vec(),
                false, /* add all currencies */
            ),
        )
    }

    fn p2p_script(&self) -> libra_types::transaction::TransactionPayload {
        let receiver = self.auth_key.derived_address();
        libra_types::transaction::TransactionPayload::Script(
            transaction_builder_generated::stdlib::encode_peer_to_peer_with_metadata_script(
                self.currency_code(),
                receiver,
                self.amount,
                vec![],
                vec![],
            ),
        )
    }
}

pub struct Service {
    chain_id: libra_types::chain_id::ChainId,
    private_key: libra_crypto::ed25519::Ed25519PrivateKey,
    client: libra_json_rpc_client::JsonRpcAsyncClient,
}

impl Service {
    pub fn new(
        server_url: String,
        chain_id: libra_types::chain_id::ChainId,
        private_key_file: String,
    ) -> Self {
        let url = reqwest::Url::parse(server_url.as_str()).expect("invalid server url");
        let private_key = generate_key::load_key(private_key_file);
        let client = libra_json_rpc_client::JsonRpcAsyncClient::new(url);
        Service {
            chain_id,
            private_key,
            client,
        }
    }

    pub async fn process(&self, params: &MintParams) -> Result<Response> {
        let (tc_seq, dd_seq) = self.sequences().await?;

        let create_account_txn = self.create_txn(
            params.create_parent_vasp_account_script(tc_seq),
            treasury_compliance_account_address(),
            tc_seq,
        )?;
        let transfer_txn =
            self.create_txn(params.p2p_script(), testnet_dd_account_address(), dd_seq)?;

        let mut batch = libra_json_rpc_client::JsonRpcBatch::new();
        batch.add_submit_request(create_account_txn.clone())?;
        batch.add_submit_request(transfer_txn.clone())?;
        self.client.execute(batch).await?;

        if let Some(return_txns) = params.return_txns {
            if return_txns {
                return Ok(Response::SubmittedTxns(vec![
                    create_account_txn,
                    transfer_txn,
                ]));
            }
        }
        Ok(Response::DDAccountNextSeqNum(dd_seq + 1))
    }

    fn create_txn(
        &self,
        payload: libra_types::transaction::TransactionPayload,
        sender: libra_types::account_address::AccountAddress,
        seq: u64,
    ) -> Result<libra_types::transaction::SignedTransaction> {
        libra_types::transaction::helpers::create_user_txn(
            self,
            payload,
            sender,
            seq,
            1_000_000,
            0,
            LBR_NAME.to_owned(),
            30,
            self.chain_id,
        )
    }

    async fn sequences(&self) -> Result<(u64, u64)> {
        let accounts = vec![
            treasury_compliance_account_address(),
            testnet_dd_account_address(),
        ];
        let responses = self.client.get_accounts(&accounts).await?;

        let treasury_compliance = responses
            .get(0)
            .as_ref()
            .ok_or_else(|| {
                anyhow::format_err!("get treasury compliance account response not found")
            })?
            .as_ref()
            .ok_or_else(|| anyhow::format_err!("treasury compliance account not found"))?
            .sequence_number;
        let designated_dealer = responses
            .get(1)
            .as_ref()
            .ok_or_else(|| anyhow::format_err!("get designated dealer account response not found"))?
            .as_ref()
            .ok_or_else(|| anyhow::format_err!("designated dealer account not found"))?
            .sequence_number;
        Ok((treasury_compliance, designated_dealer))
    }
}

impl libra_types::transaction::helpers::TransactionSigner for Service {
    fn sign_txn(
        &self,
        raw_txn: libra_types::transaction::RawTransaction,
    ) -> Result<libra_types::transaction::SignedTransaction> {
        let signature = self.private_key.sign(&raw_txn);
        Ok(libra_types::transaction::SignedTransaction::new(
            raw_txn,
            libra_crypto::ed25519::Ed25519PublicKey::from(&self.private_key),
            signature,
        ))
    }
}
