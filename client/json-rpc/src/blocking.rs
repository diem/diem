// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{process_batch_response, JsonRpcBatch, JsonRpcResponse};
use anyhow::{ensure, format_err, Result};
use reqwest::{
    blocking::{Client, ClientBuilder},
    Url,
};
use std::time::Duration;

const JSON_RPC_TIMEOUT_MS: u64 = 5_000;
const MAX_JSON_RPC_RETRY_COUNT: u64 = 2;

pub struct JsonRpcClient {
    url: Url,
    client: Client,
}

impl JsonRpcClient {
    pub fn new(url: Url) -> Result<Self> {
        Ok(Self {
            client: ClientBuilder::new().use_rustls_tls().build()?,
            url,
        })
    }

    /// Sends a JSON RPC batched request.
    /// Returns a vector of responses s.t. response order matches the request order
    pub fn execute(&mut self, batch: JsonRpcBatch) -> Result<Vec<Result<JsonRpcResponse>>> {
        if batch.requests.is_empty() {
            return Ok(vec![]);
        }
        let request = batch.json_request();

        //retry send
        let response = self
            .send_with_retry(request)?
            .error_for_status()
            .map_err(|e| format_err!("Server returned error: {:?}", e))?;

        let response = process_batch_response(batch.clone(), response.json()?)?;
        ensure!(
            batch.requests.len() == response.len(),
            "received unexpected number of responses in batch"
        );
        Ok(response)
    }

    // send with retry
    pub fn send_with_retry(
        &mut self,
        request: serde_json::Value,
    ) -> Result<reqwest::blocking::Response> {
        let mut response = self.send(&request);
        let mut try_cnt = 0;

        // retry if send fails
        while try_cnt < MAX_JSON_RPC_RETRY_COUNT && response.is_err() {
            response = self.send(&request);
            try_cnt += 1;
        }
        response
    }

    fn send(&mut self, request: &serde_json::Value) -> Result<reqwest::blocking::Response> {
        self.client
            .post(self.url.clone())
            .json(request)
            .timeout(Duration::from_millis(JSON_RPC_TIMEOUT_MS))
            .send()
            .map_err(Into::into)
    }
}
