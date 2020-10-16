// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::{
    defaults, types as jsonrpc, Error, JsonRpcResponse, RetryStrategy, State,
    WaitForTransactionError,
};
use libra_crypto::hash::CryptoHash;
use libra_types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, Transaction},
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::time::Duration;

#[derive(Debug)]
pub struct Response<R> {
    pub result: R,
    pub state: State,
}

impl<R> std::ops::Deref for Response<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

#[derive(Debug)]
pub struct Client<R> {
    pub http_client: reqwest::Client,
    pub server_url: reqwest::Url,
    pub last_known_state: std::sync::RwLock<Option<State>>,
    pub retry: R,
}

impl<R: RetryStrategy> Client<R> {
    pub fn from_url<T: reqwest::IntoUrl>(server_url: T, retry: R) -> Result<Self, reqwest::Error> {
        Ok(Self {
            http_client: reqwest::ClientBuilder::new()
                .use_native_tls()
                .timeout(defaults::HTTP_REQUEST_TIMEOUT)
                .build()
                .expect("Unable to build Client."),
            server_url: server_url.into_url()?,
            last_known_state: std::sync::RwLock::new(None),
            retry,
        })
    }

    pub async fn wait_for_signed_transaction(
        &self,
        txn: &SignedTransaction,
        timeout: Option<Duration>,
        delay: Option<Duration>,
    ) -> Result<Response<jsonrpc::Transaction>, WaitForTransactionError> {
        self.wait_for_transaction(
            &txn.sender(),
            txn.sequence_number(),
            txn.expiration_timestamp_secs(),
            &Transaction::UserTransaction(txn.clone()).hash().to_hex(),
            timeout,
            delay,
        )
        .await
    }

    pub async fn wait_for_transaction(
        &self,
        address: &AccountAddress,
        seq: u64,
        expiration_time_secs: u64,
        txn_hash: &str,
        timeout: Option<Duration>,
        delay: Option<Duration>,
    ) -> Result<Response<jsonrpc::Transaction>, WaitForTransactionError> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout.unwrap_or(defaults::TIMEOUT) {
            let txn_resp = self
                .get_account_transaction(address, seq, true)
                .await
                .map_err(WaitForTransactionError::GetTransactionError)?;
            if let Some(txn) = txn_resp.result {
                if txn.hash != txn_hash {
                    return Err(WaitForTransactionError::TransactionHashMismatchError(txn));
                }
                if let Some(ref vm_status) = txn.vm_status {
                    if vm_status.r#type != jsonrpc::VM_STATUS_EXECUTED {
                        return Err(WaitForTransactionError::TransactionExecutionFailed(txn));
                    }
                }
                return Ok(Response {
                    result: txn,
                    state: txn_resp.state,
                });
            }
            if let Some(state) = self.last_known_state() {
                if expiration_time_secs <= state.timestamp_usecs / 1_000_000 {
                    return Err(WaitForTransactionError::TransactionExpired(
                        state.timestamp_usecs,
                    ));
                }
            }
            tokio::time::delay_for(delay.unwrap_or(defaults::WAIT_DELAY)).await;
        }
        Err(WaitForTransactionError::Timeout(start.elapsed()))
    }

    pub fn last_known_state(&self) -> Option<State> {
        let data = self.last_known_state.read().unwrap();
        data.clone()
    }

    pub async fn get_metadata(&self) -> Result<Response<jsonrpc::Metadata>, Error> {
        self.send("get_metadata", json!([])).await
    }

    pub async fn get_metadata_by_version(
        &self,
        version: u64,
    ) -> Result<Response<jsonrpc::Metadata>, Error> {
        self.send("get_metadata", json!([version])).await
    }

    pub async fn get_account(
        &self,
        address: &AccountAddress,
    ) -> Result<Response<Option<jsonrpc::Account>>, Error> {
        self.send_opt("get_account", json!([address])).await
    }

    pub async fn get_account_transaction(
        &self,
        address: &AccountAddress,
        seq: u64,
        include_events: bool,
    ) -> Result<Response<Option<jsonrpc::Transaction>>, Error> {
        self.send_opt(
            "get_account_transaction",
            json!([address, seq, include_events]),
        )
        .await
    }

    pub async fn get_account_transactions(
        &self,
        address: &AccountAddress,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Vec<jsonrpc::Transaction>>, Error> {
        self.send(
            "get_account_transactions",
            json!([address, start_seq, limit, include_events]),
        )
        .await
    }

    pub async fn get_transactions(
        &self,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Vec<jsonrpc::Transaction>>, Error> {
        self.send(
            "get_transactions",
            json!([start_seq, limit, include_events]),
        )
        .await
    }

    pub async fn get_events(
        &self,
        key: &str,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<jsonrpc::Event>>, Error> {
        self.send("get_events", json!([key, start_seq, limit]))
            .await
    }

    pub async fn get_currencies(&self) -> Result<Response<Vec<jsonrpc::CurrencyInfo>>, Error> {
        self.send("get_currencies", json!([])).await
    }

    pub async fn get_state_proof(
        &self,
        from_version: u64,
    ) -> Result<Response<jsonrpc::StateProof>, Error> {
        self.send("get_state_proof", json!([from_version])).await
    }

    pub async fn get_account_state_with_proof(
        &self,
        address: &AccountAddress,
        from_version: Option<u64>,
        to_version: Option<u64>,
    ) -> Result<Response<jsonrpc::AccountStateWithProof>, Error> {
        self.send(
            "get_account_state_with_proof",
            json!([address, from_version, to_version]),
        )
        .await
    }

    pub async fn submit(&self, txn: &SignedTransaction) -> Result<Response<()>, Error> {
        let txn_payload = hex::encode(lcs::to_bytes(&txn).map_err(Error::unexpected_lcs_error)?);
        let resp = self
            .send_without_retry("submit", &json!([txn_payload]))
            .await?;
        Ok(Response {
            result: (),
            state: State::from_response(&resp),
        })
    }

    pub async fn send<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response<T>, Error> {
        let resp = self.send_with_retry(method, params).await?;
        let state = State::from_response(&resp);
        match resp.result {
            Some(ret) => Ok(Response {
                result: serde_json::from_value(ret).map_err(Error::DeserializeResponseJsonError)?,
                state,
            }),
            None => Err(Error::ResultNotFound(resp)),
        }
    }

    pub async fn send_opt<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response<Option<T>>, Error> {
        let resp = self.send_with_retry(method, params).await?;
        let state = State::from_response(&resp);
        let result = match resp.result {
            Some(ret) => {
                Some(serde_json::from_value(ret).map_err(Error::DeserializeResponseJsonError)?)
            }
            None => None,
        };
        Ok(Response { result, state })
    }

    pub async fn send_with_retry(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<JsonRpcResponse, Error> {
        let mut retries: u32 = 0;
        loop {
            let ret = self.send_without_retry(method, &params).await;
            match ret {
                Ok(r) => {
                    return Ok(r);
                }
                Err(err) => {
                    if !self.retry.is_retriable(&err) {
                        return Err(err);
                    }
                    if retries < self.retry.max_retries(&err) {
                        match retries.checked_add(1) {
                            Some(i) if i < self.retry.max_retries(&err) => {
                                retries = i;
                            }
                            _ => return Err(err),
                        };
                        tokio::time::delay_for(self.retry.delay(&err, retries)).await;
                        continue;
                    }
                    return Err(err);
                }
            }
        }
    }

    pub async fn send_without_retry(
        &self,
        method: &str,
        params: &serde_json::Value,
    ) -> Result<JsonRpcResponse, Error> {
        let resp = self
            .http_client
            .post(self.server_url.clone())
            .json(&json!({"jsonrpc": "2.0", "method": method, "params": params, "id": 1}))
            .send()
            .await
            .map_err(Error::NetworkError)?;
        if resp.status() != 200 {
            return Err(Error::InvalidHTTPStatus(format!("{:#?}", resp)));
        }
        let rpc_resp: JsonRpcResponse = resp.json().await.map_err(Error::InvalidHTTPResponse)?;

        if rpc_resp.jsonrpc != "2.0" {
            return Err(Error::InvalidRpcResponse(rpc_resp));
        }

        if let Some(state) = self.last_known_state() {
            if rpc_resp.libra_chain_id != state.chain_id {
                return Err(Error::ChainIdMismatch(rpc_resp));
            }
        }

        let resp_state = State::from_response(&rpc_resp);
        if !self.update_state(resp_state) {
            return Err(Error::StaleResponseError(rpc_resp));
        }

        if let Some(err) = rpc_resp.error {
            return Err(Error::JsonRpcError(err));
        }

        Ok(rpc_resp)
    }

    pub fn update_state(&self, resp_state: State) -> bool {
        let mut state_writer = self.last_known_state.write().unwrap();
        if let Some(state) = &*state_writer {
            if &resp_state < state {
                return false;
            }
        }
        *state_writer = Some(resp_state);
        true
    }
}
