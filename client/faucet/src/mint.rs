// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_crypto::traits::SigningKey;
use diem_types::account_config::{
    testnet_dd_account_address, treasury_compliance_account_address, type_tag_for_currency_code,
    XUS_NAME,
};
use std::{convert::From, fmt};

#[derive(Debug)]
pub enum Response {
    DDAccountNextSeqNum(u64),
    SubmittedTxns(Vec<diem_types::transaction::SignedTransaction>),
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::DDAccountNextSeqNum(v1) => write!(f, "{}", v1),
            Response::SubmittedTxns(v2) => {
                write!(f, "{}", hex::encode(bcs::to_bytes(&v2).unwrap()))
            }
        }
    }
}

#[derive(serde_derive::Deserialize)]
pub struct MintParams {
    pub amount: u64,
    pub currency_code: move_core_types::identifier::Identifier,
    pub auth_key: diem_types::transaction::authenticator::AuthenticationKey,
    pub return_txns: Option<bool>,
    pub is_designated_dealer: Option<bool>,
}

impl MintParams {
    fn currency_code(&self) -> move_core_types::language_storage::TypeTag {
        type_tag_for_currency_code(self.currency_code.to_owned())
    }

    fn create_parent_vasp_account_script(
        &self,
        seq: u64,
    ) -> diem_types::transaction::TransactionPayload {
        diem_types::transaction::TransactionPayload::Script(
            transaction_builder_generated::stdlib::encode_create_parent_vasp_account_script(
                self.currency_code(),
                0, // sliding nonce
                self.receiver(),
                self.auth_key.prefix().to_vec(),
                format!("No. {} VASP", seq).as_bytes().to_vec(),
                false, /* add all currencies */
            ),
        )
    }

    fn create_designated_dealer_account_script(
        &self,
        seq: u64,
    ) -> diem_types::transaction::TransactionPayload {
        diem_types::transaction::TransactionPayload::Script(
            transaction_builder_generated::stdlib::encode_create_designated_dealer_script(
                self.currency_code(),
                0, // sliding nonce
                self.receiver(),
                self.auth_key.prefix().to_vec(),
                format!("No. {} DD", seq).as_bytes().to_vec(),
                false, /* add all currencies */
            ),
        )
    }

    fn p2p_script(&self) -> diem_types::transaction::TransactionPayload {
        diem_types::transaction::TransactionPayload::Script(
            transaction_builder_generated::stdlib::encode_peer_to_peer_with_metadata_script(
                self.currency_code(),
                self.receiver(),
                self.amount,
                vec![],
                vec![],
            ),
        )
    }

    fn receiver(&self) -> diem_types::account_address::AccountAddress {
        self.auth_key.derived_address()
    }
}

pub struct Service {
    chain_id: diem_types::chain_id::ChainId,
    private_key: diem_crypto::ed25519::Ed25519PrivateKey,
    client: diem_json_rpc_client::JsonRpcAsyncClient,
}

impl Service {
    pub fn new(
        server_url: String,
        chain_id: diem_types::chain_id::ChainId,
        private_key_file: String,
    ) -> Self {
        let url = reqwest::Url::parse(server_url.as_str()).expect("invalid server url");
        let private_key = generate_key::load_key(private_key_file);
        let client = diem_json_rpc_client::JsonRpcAsyncClient::new(url);
        Service {
            chain_id,
            private_key,
            client,
        }
    }

    pub async fn process(&self, params: &MintParams) -> Result<Response> {
        let (tc_seq, dd_seq, receiver_seq) = self.sequences(params.receiver()).await?;

        let mut txns = vec![];
        if receiver_seq.is_none() {
            let script = if params.is_designated_dealer.unwrap_or(false) {
                params.create_designated_dealer_account_script(tc_seq)
            } else {
                params.create_parent_vasp_account_script(tc_seq)
            };
            txns.push(self.create_txn(script, treasury_compliance_account_address(), tc_seq)?);
        }
        txns.push(self.create_txn(params.p2p_script(), testnet_dd_account_address(), dd_seq)?);

        let mut batch = diem_json_rpc_client::JsonRpcBatch::new();
        for txn in &txns {
            batch.add_submit_request(txn.clone())?;
        }
        self.client.execute(batch).await?;

        if let Some(return_txns) = params.return_txns {
            if return_txns {
                return Ok(Response::SubmittedTxns(txns));
            }
        }
        Ok(Response::DDAccountNextSeqNum(dd_seq + 1))
    }

    fn create_txn(
        &self,
        payload: diem_types::transaction::TransactionPayload,
        sender: diem_types::account_address::AccountAddress,
        seq: u64,
    ) -> Result<diem_types::transaction::SignedTransaction> {
        diem_types::transaction::helpers::create_user_txn(
            self,
            payload,
            sender,
            seq,
            1_000_000,
            0,
            XUS_NAME.to_owned(),
            30,
            self.chain_id,
        )
    }

    async fn sequences(
        &self,
        receiver: diem_types::account_address::AccountAddress,
    ) -> Result<(u64, u64, Option<u64>)> {
        let accounts = vec![
            treasury_compliance_account_address(),
            testnet_dd_account_address(),
            receiver,
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
        let receiver = responses
            .get(2)
            .as_ref()
            .ok_or_else(|| anyhow::format_err!("get receiver account response not found"))?
            .as_ref();
        let receiver_seq_num = if let Some(account) = receiver {
            Some(account.sequence_number)
        } else {
            None
        };
        Ok((treasury_compliance, designated_dealer, receiver_seq_num))
    }
}

impl diem_types::transaction::helpers::TransactionSigner for Service {
    fn sign_txn(
        &self,
        raw_txn: diem_types::transaction::RawTransaction,
    ) -> Result<diem_types::transaction::SignedTransaction> {
        let signature = self.private_key.sign(&raw_txn);
        Ok(diem_types::transaction::SignedTransaction::new(
            raw_txn,
            diem_crypto::ed25519::Ed25519PublicKey::from(&self.private_key),
            signature,
        ))
    }
}
