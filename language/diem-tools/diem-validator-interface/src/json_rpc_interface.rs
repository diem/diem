// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::DiemValidatorInterface;
use anyhow::{bail, Result};
use diem_json_rpc_client::{JsonRpcBatch, JsonRpcClient, JsonRpcResponse};
use diem_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    transaction::{Transaction, Version},
};
use reqwest::Url;

pub struct JsonRpcDebuggerInterface {
    client: JsonRpcClient,
}

impl JsonRpcDebuggerInterface {
    pub fn new(url: &str) -> Result<Self> {
        let url = Url::parse(url)?;
        Ok(Self {
            client: JsonRpcClient::new(url)?,
        })
    }

    fn execute_single_command(&self, cmd: JsonRpcBatch) -> Result<JsonRpcResponse> {
        let mut ret = self.client.execute(cmd)?;
        if ret.len() != 1 {
            bail!("Unexpected response length")
        }
        ret.pop().unwrap()
    }
}

impl DiemValidatorInterface for JsonRpcDebuggerInterface {
    fn get_account_state_by_version(
        &self,
        account: AccountAddress,
        version: Version,
    ) -> Result<Option<AccountStateBlob>> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_account_state_with_proof_request(account, Some(version), None);

        let resp = self.execute_single_command(batch)?;
        if let JsonRpcResponse::AccountStateWithProofResponse(account_state) = resp {
            Ok(match account_state.blob {
                Some(bytes) => Some(bcs::from_bytes(&bytes.into_bytes()?)?),
                None => None,
            })
        } else {
            bail!("Unexpected response type");
        }
    }

    fn get_committed_transactions(&self, start: Version, limit: u64) -> Result<Vec<Transaction>> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_transactions_request(start, limit, false);

        let resp = self.execute_single_command(batch)?;
        if let JsonRpcResponse::TransactionsResponse(txns) = resp {
            let mut output = vec![];
            for txn in txns.into_iter() {
                let raw_bytes = txn.bytes.into_bytes()?;
                output.push(bcs::from_bytes(&raw_bytes)?);
            }
            Ok(output)
        } else {
            bail!("Unexpected response type");
        }
    }

    fn get_latest_version(&self) -> Result<Version> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_metadata_request(None);

        let resp = self.execute_single_command(batch)?;
        if let JsonRpcResponse::MetadataViewResponse(metadata) = resp {
            Ok(metadata.version)
        } else {
            bail!("Unexpected response type");
        }
    }

    fn get_version_by_account_sequence(
        &self,
        account: AccountAddress,
        seq: u64,
    ) -> Result<Option<Version>> {
        let mut batch = JsonRpcBatch::new();
        batch.add_get_account_transaction_request(account, seq, false);

        let resp = self.execute_single_command(batch)?;
        if let JsonRpcResponse::AccountTransactionResponse(metadata) = resp {
            Ok(metadata.map(|txn| txn.version))
        } else {
            bail!("Unexpected response type");
        }
    }
}
