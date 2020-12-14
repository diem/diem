// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{errors::JsonRpcError, views::AccountView, JsonRpcResponse};
use anyhow::{ensure, format_err, Error, Result};
use diem_types::{account_address::AccountAddress, transaction::SignedTransaction};
use reqwest::{Client, ClientBuilder, StatusCode, Url};
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
        let txn_payload = hex::encode(bcs::to_bytes(&transaction)?);
        self.add_request("submit".to_string(), vec![Value::String(txn_payload)]);
        Ok(())
    }

    pub fn add_get_account_request(&mut self, address: AccountAddress) {
        self.add_request(
            "get_account".to_string(),
            vec![Value::String(address.to_string())],
        );
    }

    pub fn add_get_metadata_request(&mut self, version: Option<u64>) {
        let params = match version {
            Some(version) => vec![json!(version)],
            None => vec![],
        };
        self.add_request("get_metadata".to_string(), params)
    }

    pub fn add_get_currencies_info(&mut self) {
        self.add_request("get_currencies".to_string(), vec![]);
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

    pub fn add_get_account_transactions_request(
        &mut self,
        account: AccountAddress,
        start: u64,
        limit: u64,
        include_events: bool,
    ) {
        self.add_request(
            "get_account_transactions".to_string(),
            vec![
                json!(account.to_string()),
                json!(start),
                json!(limit),
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

    pub fn add_get_network_status_request(&mut self) {
        self.add_request("get_network_status".to_string(), vec![]);
    }
}

#[derive(Debug)]
pub enum JsonRpcAsyncClientError {
    // Errors surfaced by the http client - connection errors, etc
    ClientError(reqwest::Error),
    // Error constructing the request
    InvalidArgument(String),
    // Failed to parse response from server
    InvalidServerResponse(String),
    // Received a non 200 OK HTTP Code
    HTTPError(StatusCode),
    // An error code returned by the JSON RPC Server
    JsonRpcError(JsonRpcError),
}

impl JsonRpcAsyncClientError {
    pub fn is_retriable(&self) -> bool {
        match self {
            JsonRpcAsyncClientError::ClientError(e) => {
                if e.is_timeout() || e.is_request() {
                    return true;
                }
                if let Some(status) = e.status() {
                    // Returned status code indicates a server error
                    return status.is_server_error();
                }
                false
            }
            JsonRpcAsyncClientError::HTTPError(status) => status.is_server_error(),
            _ => false,
        }
    }
}

impl std::convert::From<JsonRpcAsyncClientError> for anyhow::Error {
    fn from(e: JsonRpcAsyncClientError) -> Self {
        anyhow::Error::msg(e.to_string())
    }
}

impl std::fmt::Display for JsonRpcAsyncClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            JsonRpcAsyncClientError::InvalidArgument(e) => write!(f, "InvalidArgument: {}", e),
            JsonRpcAsyncClientError::ClientError(e) => write!(f, "ClientError: {}", e.to_string()),
            JsonRpcAsyncClientError::InvalidServerResponse(e) => {
                write!(f, "InvalidServerResponse {}", e)
            }
            JsonRpcAsyncClientError::JsonRpcError(e) => write!(f, "JsonRpcError {}", e),
            JsonRpcAsyncClientError::HTTPError(e) => write!(f, "HTTPError. Status Code: {}", e),
        }
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

    pub async fn get_accounts(
        &self,
        accounts: &[AccountAddress],
    ) -> Result<Vec<Option<AccountView>>, JsonRpcAsyncClientError> {
        let mut batch = JsonRpcBatch::new();
        for account in accounts {
            batch.add_get_account_request(*account);
        }
        let exec_results = self.execute(batch).await?;
        let mut results = vec![];
        for exec_result in exec_results {
            let exec_result = exec_result
                .map_err(|e| JsonRpcAsyncClientError::InvalidServerResponse(e.to_string()))?;
            if let JsonRpcResponse::AccountResponse(r) = exec_result {
                results.push(r);
            } else {
                panic!("Unexpected response for get_accounts {:?}", exec_result)
            }
        }

        if results.len() != accounts.len() {
            return Err(JsonRpcAsyncClientError::InvalidServerResponse(format!(
                "Received unexpected number of JSON RPC responses ({}) for {} requests",
                results.len(),
                accounts.len()
            )));
        }

        Ok(results)
    }

    pub async fn submit_transaction(
        &self,
        txn: SignedTransaction,
    ) -> Result<(), JsonRpcAsyncClientError> {
        let mut batch = JsonRpcBatch::new();
        batch
            .add_submit_request(txn)
            .map_err(|e| JsonRpcAsyncClientError::InvalidArgument(e.to_string()))?;
        let mut exec_result = self.execute(batch).await?;
        assert!(exec_result.len() == 1);
        exec_result
            .remove(0)
            .map(|_| ())
            .map_err(|e| JsonRpcAsyncClientError::InvalidServerResponse(e.to_string()))
    }

    pub async fn execute(
        &self,
        batch: JsonRpcBatch,
    ) -> Result<Vec<Result<JsonRpcResponse>>, JsonRpcAsyncClientError> {
        let requests = batch.json_request();
        let resp = self
            .client
            .post(&self.address)
            .json(&requests)
            .send()
            .await
            .map_err(JsonRpcAsyncClientError::ClientError)?;

        if resp.status() != 200 {
            return Err(JsonRpcAsyncClientError::HTTPError(resp.status()));
        }

        let json: Value = resp
            .json()
            .await
            .map_err(|e| JsonRpcAsyncClientError::InvalidServerResponse(e.to_string()))?;

        if let Value::Array(responses) = json {
            process_batch_response(batch, responses)
                .map_err(|e| JsonRpcAsyncClientError::InvalidServerResponse(e.to_string()))
        } else if let Value::Object(response) = json {
            let error_value = response.get("error").ok_or_else(|| {
                JsonRpcAsyncClientError::InvalidServerResponse(
                    "batch response should be an array".to_string(),
                )
            })?;
            let rpc_err: JsonRpcError = serde_json::from_value(error_value.clone())
                .map_err(|e| JsonRpcAsyncClientError::InvalidServerResponse(e.to_string()))?;
            Err(JsonRpcAsyncClientError::JsonRpcError(rpc_err))
        } else {
            Err(JsonRpcAsyncClientError::InvalidServerResponse(
                json.to_string(),
            ))
        }
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
        .map(|_| Err(format_err!("request is missing")))
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use reqwest::{blocking::ClientBuilder, Url};

    #[test]
    fn test_error_is_retriable() {
        let http_status_code_err = JsonRpcAsyncClientError::HTTPError(StatusCode::BAD_GATEWAY);
        let server_err = JsonRpcAsyncClientError::InvalidServerResponse(
            "test invalid server response".to_string(),
        );
        // Reqwest error's builder is private to the crate, so send out a
        // fake request that should fail to get an error
        let test_client = ClientBuilder::new()
            .timeout(Duration::from_millis(1))
            .build()
            .unwrap();
        let req_err = test_client
            .get(Url::parse("http://192.108.0.1").unwrap())
            .send()
            .unwrap_err();
        let client_err = JsonRpcAsyncClientError::ClientError(req_err);
        let arg_err = JsonRpcAsyncClientError::InvalidArgument("test invalid argument".to_string());
        // Make sure display is implemented correctly for error enum
        println!("{}", client_err);
        assert_eq!(server_err.is_retriable(), false);
        assert_eq!(http_status_code_err.is_retriable(), true);
        assert_eq!(arg_err.is_retriable(), false);
        assert_eq!(client_err.is_retriable(), true);
    }
}
