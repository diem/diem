// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{errors::JsonRpcError, views::AccountView, JsonRpcResponse};
use anyhow::{ensure, format_err, Error, Result};
use libra_types::{account_address::AccountAddress, transaction::SignedTransaction};
use reqwest::{Client, ClientBuilder, Url};
use serde_json::{json, Value};
use std::{collections::HashSet, convert::TryFrom, fmt, time::Duration};

#[derive(Clone, Default)]
pub struct JsonRpcBatch {
    pub requests: Vec<(String, Vec<Value>)>,
}

impl JsonRpcBatch {
    pub fn new() -> Self {
        Self { requests: vec![] }
    }

    pub fn json_request(&self) -> Value {
        let requests = self
            .requests
            .iter()
            .enumerate()
            .map(|(id, (method, params))| {
                serde_json::json!({"jsonrpc": "2.0", "method": method, "params": params, "id": id})
            })
            .collect();
        serde_json::Value::Array(requests)
    }

    pub fn add_request(&mut self, method_name: String, parameters: Vec<Value>) {
        self.requests.push((method_name, parameters));
    }

    pub fn add_submit_request(&mut self, transaction: SignedTransaction) -> Result<()> {
        let txn_payload = hex::encode(lcs::to_bytes(&transaction)?);
        self.add_request("submit".to_string(), vec![Value::String(txn_payload)]);
        Ok(())
    }

    pub fn add_get_account_state_request(&mut self, address: AccountAddress) {
        self.add_request(
            "get_account_state".to_string(),
            vec![Value::String(address.to_string())],
        );
    }

    pub fn add_get_metadata_request(&mut self) {
        self.add_request("get_metadata".to_string(), vec![]);
    }

    pub fn add_get_currencies_info(&mut self) {
        self.add_request("currencies_info".to_string(), vec![]);
    }

    pub fn add_get_transactions_request(
        &mut self,
        start_version: u64,
        limit: u64,
        include_events: bool,
    ) {
        self.add_request(
            "get_transactions".to_string(),
            vec![json!(start_version), json!(limit), json!(include_events)],
        );
    }

    pub fn add_get_account_transaction_request(
        &mut self,
        account: AccountAddress,
        sequence: u64,
        include_events: bool,
    ) {
        self.add_request(
            "get_account_transaction".to_string(),
            vec![
                json!(account.to_string()),
                json!(sequence),
                json!(include_events),
            ],
        );
    }

    pub fn add_get_events_request(&mut self, event_key: String, start: u64, limit: u64) {
        self.add_request(
            "get_events".to_string(),
            vec![json!(event_key), json!(start), json!(limit)],
        );
    }

    pub fn add_get_state_proof_request(&mut self, known_version: u64) {
        self.add_request("get_state_proof".to_string(), vec![json!(known_version)]);
    }

    pub fn add_get_account_state_with_proof_request(
        &mut self,
        account: AccountAddress,
        version: Option<u64>,
        ledger_version: Option<u64>,
    ) {
        self.add_request(
            "get_account_state_with_proof".to_string(),
            vec![
                json!(account.to_string()),
                json!(version),
                json!(ledger_version),
            ],
        );
    }
}

#[derive(Clone)]
pub struct JsonRpcAsyncClient {
    address: String,
    client: Client,
}

impl JsonRpcAsyncClient {
    /// Pass in full url for endpoint, supports HTTPS
    pub fn new(url: Url) -> Self {
        Self {
            address: url.to_string(),
            client: ClientBuilder::new()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Unable to build Client."),
        }
    }

    pub fn new_with_client(client: Client, url: Url) -> Self {
        Self {
            address: url.to_string(),
            client,
        }
    }

    pub async fn get_accounts_state(
        &self,
        accounts: &[AccountAddress],
    ) -> Result<Vec<Option<AccountView>>> {
        let mut batch = JsonRpcBatch::new();
        for account in accounts {
            batch.add_get_account_state_request(*account);
        }
        let exec_results = self.execute(batch).await?;
        let mut results = vec![];
        for exec_result in exec_results {
            let exec_result = exec_result?;
            if let JsonRpcResponse::AccountResponse(r) = exec_result {
                results.push(r);
            } else {
                panic!(
                    "Unexpected response for get_accounts_state {:?}",
                    exec_result
                )
            }
        }
        ensure!(
            results.len() == accounts.len(),
            "Received unexpected number of JSON RPC responses ({}) for {} requests",
            results.len(),
            accounts.len()
        );
        Ok(results)
    }

    pub async fn submit_transaction(&self, txn: SignedTransaction) -> Result<()> {
        let mut batch = JsonRpcBatch::new();
        batch.add_submit_request(txn)?;
        let mut exec_result = self.execute(batch).await?;
        assert!(exec_result.len() == 1);
        exec_result.remove(0).map(|_| ())
    }

    pub async fn execute(&self, batch: JsonRpcBatch) -> Result<Vec<Result<JsonRpcResponse>>> {
        let requests = batch.json_request();
        let resp = self
            .client
            .post(&self.address)
            .json(&requests)
            .send()
            .await?;

        ensure!(
            resp.status() == 200,
            format!("Http error code {}", resp.status())
        );

        let responses: Vec<Value> = resp.json().await?;
        process_batch_response(batch, responses)
    }
}

impl fmt::Debug for JsonRpcAsyncClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

////////////////////////////////////////////////////
/// Helper methods for basic payload processing ///
///////////////////////////////////////////////////

pub fn process_batch_response(
    batch: JsonRpcBatch,
    responses: Vec<Value>,
) -> Result<Vec<Result<JsonRpcResponse>>> {
    let mut result = batch
        .requests
        .iter()
        .map(|_| Err(format_err!("response is missing")))
        .collect::<Vec<_>>();

    let mut seen_ids = HashSet::new();
    for response in responses {
        // check JSON RPC protocol
        let json_rpc_protocol = response.get("jsonrpc");
        ensure!(
            json_rpc_protocol == Some(&json!("2.0")),
            "JSON RPC response with incorrect protocol: {:?}",
            json_rpc_protocol
        );

        if let Ok(req_id) = fetch_id(&response) {
            ensure!(
                !seen_ids.contains(&req_id),
                "received JSON RPC response with duplicate response ID"
            );
            seen_ids.insert(req_id);
            if req_id < result.len() {
                let resp = if let Some(err_data) = response.get("error") {
                    let err_json: JsonRpcError = serde_json::from_value(err_data.clone())?;
                    Err(Error::new(err_json))
                } else if let Some(data) = response.get("result") {
                    let method = batch.requests[req_id].0.clone();
                    Ok(JsonRpcResponse::try_from((method, data.clone()))?)
                } else {
                    continue;
                };
                result[req_id] = resp;
            }
        }
    }

    Ok(result)
}

pub fn get_response_from_batch(
    index: usize,
    batch: &[Result<JsonRpcResponse>],
) -> Result<&Result<JsonRpcResponse>> {
    batch.get(index).ok_or_else(|| {
        format_err!(
            "[JSON RPC client] response missing in batch at index {}",
            index
        )
    })
}

fn fetch_id(response: &Value) -> Result<usize> {
    match response.get("id") {
        Some(id) => Ok(serde_json::from_value::<usize>(id.clone())?),
        None => Err(format_err!("request id is missing")),
    }
}
