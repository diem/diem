// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::views::AccountView;
use anyhow::{ensure, format_err, Error, Result};
use libra_types::{account_address::AccountAddress, transaction::SignedTransaction};
use reqwest::Client;
use serde_json::Value;
use std::convert::TryFrom;

#[derive(Default)]
pub struct JsonRpcBatch {
    pub requests: Vec<(String, Vec<Value>)>,
}

impl JsonRpcBatch {
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
}

pub struct JsonRpcAsyncClient {
    address: String,
    client: Client,
}

impl JsonRpcAsyncClient {
    pub fn new(host: &str, port: u16) -> Self {
        let address = format!("http://{}:{}", host, port);
        let client = Client::new();
        Self { address, client }
    }

    pub async fn execute(&self, batch: JsonRpcBatch) -> Result<Vec<Result<JsonRpcResponse>>> {
        let requests = batch
            .requests
            .iter()
            .enumerate()
            .map(|(id, (method, params))| {
                serde_json::json!({"jsonrpc": "2.0", "method": method, "params": params, "id": id})
            })
            .collect();

        let resp = self
            .client
            .post(&self.address)
            .json(&serde_json::Value::Array(requests))
            .send()
            .await?;

        ensure!(
            resp.status() == 200,
            format!("Http error code {}", resp.status())
        );

        let responses: Vec<Value> = resp.json().await?;
        let mut result = vec![];
        for _ in batch.requests.iter() {
            result.push(Err(format_err!("response is missing")));
        }

        for response in responses {
            if let Ok(req_id) = self.fetch_id(&response) {
                if req_id < result.len() {
                    if let Some(err_data) = response.get("error") {
                        result[req_id] =
                            Err(format_err!("JSON-RPC error {:?}", err_data.get("message")));
                        continue;
                    }
                    if let Some(data) = response.get("result") {
                        let method = batch.requests[req_id].0.clone();
                        result[req_id] = Ok(JsonRpcResponse::try_from((method, data.clone()))?);
                    }
                }
            }
        }
        Ok(result)
    }

    fn fetch_id(&self, response: &Value) -> Result<usize> {
        match response.get("id") {
            Some(id) => Ok(serde_json::from_value::<usize>(id.clone())?),
            None => Err(format_err!("request id is missing")),
        }
    }
}

#[derive(PartialEq)]
pub enum JsonRpcResponse {
    SubmissionResponse,
    AccountResponse(AccountView),
    UnknownResponse,
}

impl TryFrom<(String, Value)> for JsonRpcResponse {
    type Error = Error;

    fn try_from((method, value): (String, Value)) -> Result<JsonRpcResponse> {
        if method == "submit" {
            Ok(JsonRpcResponse::SubmissionResponse)
        } else if method == "get_account_state" {
            let account: AccountView = serde_json::from_value(value)?;
            Ok(JsonRpcResponse::AccountResponse(account))
        } else {
            Ok(JsonRpcResponse::UnknownResponse)
        }
    }
}
