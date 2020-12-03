// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::{
    defaults, types as jsonrpc, Error, JsonRpcResponse, RetryStrategy, State,
    WaitForTransactionError,
};
use diem_crypto::hash::CryptoHash;
use diem_types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, Transaction},
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    time::Duration,
};

#[derive(Debug)]
pub struct Request {
    pub method: &'static str,
    pub params: serde_json::Value,
}

impl Request {
    pub fn new(method: &'static str, params: serde_json::Value) -> Self {
        Request { method, params }
    }

    pub fn submit(txn: &SignedTransaction) -> Result<Self, lcs::Error> {
        let txn_payload = hex::encode(lcs::to_bytes(txn)?);
        Ok(Self::new("submit", json!([txn_payload])))
    }

    pub fn get_account_state_with_proof(
        address: &AccountAddress,
        from_version: Option<u64>,
        to_version: Option<u64>,
    ) -> Self {
        Self::new(
            "get_account_state_with_proof",
            json!([address, from_version, to_version]),
        )
    }

    pub fn get_state_proof(from_version: u64) -> Self {
        Self::new("get_state_proof", json!([from_version]))
    }

    pub fn get_currencies() -> Self {
        Self::new("get_currencies", json!([]))
    }

    pub fn get_events(key: &str, start_seq: u64, limit: u64) -> Self {
        Self::new("get_events", json!([key, start_seq, limit]))
    }

    pub fn get_transactions(start_seq: u64, limit: u64, include_events: bool) -> Self {
        Self::new(
            "get_transactions",
            json!([start_seq, limit, include_events]),
        )
    }

    pub fn get_account_transactions(
        address: &AccountAddress,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Self {
        Self::new(
            "get_account_transactions",
            json!([address, start_seq, limit, include_events]),
        )
    }

    pub fn get_account_transaction(
        address: &AccountAddress,
        seq: u64,
        include_events: bool,
    ) -> Self {
        Self::new(
            "get_account_transaction",
            json!([address, seq, include_events]),
        )
    }

    pub fn get_account(address: &AccountAddress) -> Self {
        Self::new("get_account", json!([address]))
    }
    pub fn get_metadata_by_version(version: u64) -> Self {
        Self::new("get_metadata", json!([version]))
    }

    pub fn get_metadata() -> Self {
        Self::new("get_metadata", json!([]))
    }

    pub fn to_json(&self, id: usize) -> serde_json::Value {
        json!({"jsonrpc": "2.0", "id": id, "method": self.method, "params": self.params})
    }
}

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

impl<R: for<'de> serde::Deserialize<'de>> TryFrom<JsonRpcResponse> for Response<R> {
    type Error = Error;

    fn try_from(resp: JsonRpcResponse) -> Result<Self, Error> {
        let state = State::from_response(&resp);
        match resp.result {
            Some(ret) => Ok(Self {
                result: serde_json::from_value::<R>(ret)
                    .map_err(Error::DeserializeResponseJsonError)?,
                state,
            }),
            None => Err(Error::ResultNotFound(resp)),
        }
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

    pub async fn get_metadata(&self) -> Result<Response<jsonrpc::Metadata>, Error> {
        self.send(Request::get_metadata()).await
    }

    pub async fn get_metadata_by_version(
        &self,
        version: u64,
    ) -> Result<Response<jsonrpc::Metadata>, Error> {
        self.send(Request::get_metadata_by_version(version)).await
    }

    pub async fn get_account(
        &self,
        address: &AccountAddress,
    ) -> Result<Response<Option<jsonrpc::Account>>, Error> {
        self.send_opt(Request::get_account(address)).await
    }

    pub async fn get_account_transaction(
        &self,
        address: &AccountAddress,
        seq: u64,
        include_events: bool,
    ) -> Result<Response<Option<jsonrpc::Transaction>>, Error> {
        self.send_opt(Request::get_account_transaction(
            address,
            seq,
            include_events,
        ))
        .await
    }

    pub async fn get_account_transactions(
        &self,
        address: &AccountAddress,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Vec<jsonrpc::Transaction>>, Error> {
        self.send(Request::get_account_transactions(
            address,
            start_seq,
            limit,
            include_events,
        ))
        .await
    }

    pub async fn get_transactions(
        &self,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Vec<jsonrpc::Transaction>>, Error> {
        self.send(Request::get_transactions(start_seq, limit, include_events))
            .await
    }

    pub async fn get_events(
        &self,
        key: &str,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<jsonrpc::Event>>, Error> {
        self.send(Request::get_events(key, start_seq, limit)).await
    }

    pub async fn get_currencies(&self) -> Result<Response<Vec<jsonrpc::CurrencyInfo>>, Error> {
        self.send(Request::get_currencies()).await
    }

    pub async fn get_state_proof(
        &self,
        from_version: u64,
    ) -> Result<Response<jsonrpc::StateProof>, Error> {
        self.send(Request::get_state_proof(from_version)).await
    }

    pub async fn get_account_state_with_proof(
        &self,
        address: &AccountAddress,
        from_version: Option<u64>,
        to_version: Option<u64>,
    ) -> Result<Response<jsonrpc::AccountStateWithProof>, Error> {
        self.send(Request::get_account_state_with_proof(
            address,
            from_version,
            to_version,
        ))
        .await
    }

    pub async fn submit(&self, txn: &SignedTransaction) -> Result<Response<()>, Error> {
        let req = Request::submit(txn).map_err(Error::unexpected_lcs_error)?;
        let resp = self.send_without_retry(&req).await?;
        Ok(Response {
            result: (),
            state: State::from_response(&resp),
        })
    }

    pub async fn send<T: DeserializeOwned>(&self, request: Request) -> Result<Response<T>, Error> {
        self.send_with_retry(&request).await?.try_into()
    }

    pub async fn send_opt<T: DeserializeOwned>(
        &self,
        request: Request,
    ) -> Result<Response<Option<T>>, Error> {
        let resp = self.send_with_retry(&request).await?;
        let state = State::from_response(&resp);
        let result = match resp.result {
            Some(ret) => {
                Some(serde_json::from_value(ret).map_err(Error::DeserializeResponseJsonError)?)
            }
            None => None,
        };
        Ok(Response { result, state })
    }

    pub async fn send_with_retry(&self, request: &Request) -> Result<JsonRpcResponse, Error> {
        let mut retries: u32 = 0;
        loop {
            let ret = self.send_without_retry(request).await;
            match ret {
                Ok(r) => return Ok(r),
                Err(err) => retries = self.handle_retry_error(retries, err).await?,
            }
        }
    }

    pub async fn send_without_retry(&self, request: &Request) -> Result<JsonRpcResponse, Error> {
        let id = 1;
        let rpc_resp: JsonRpcResponse = self.send_json_request(request.to_json(id)).await?;
        self.validate(&rpc_resp, id, id)?;
        Ok(rpc_resp)
    }

    /// Batch requests into one JSON-RPC batch request.
    /// To keep interface simple, this method returns error when any error occurs.
    /// When batch responses contain partial success response, we will return the first
    /// error it hit.
    ///
    /// Caller can convert `JsonRpcResponse` into `Response<T>`
    /// in order of given requests:
    ///
    /// ```rust
    /// use diem_json_rpc_client::async_client::{
    ///     types as jsonrpc, Client, Error, Request, Response, Retry,
    /// };
    /// use std::convert::{TryInto};
    ///
    /// # async fn doc() -> Result<(), Error> {
    /// let client = Client::from_url("http://testnet.diem.com/v1", Retry::default()).unwrap();
    /// let mut res = client
    ///     .batch_send(vec![Request::get_metadata(), Request::get_currencies()])
    ///     .await?;
    ///
    /// let metadata: Response<jsonrpc::Metadata> = res
    ///     .remove(0)
    ///     .try_into()?;
    ///
    /// let currencies: Response<jsonrpc::CurrencyInfo> = res
    ///     .remove(0)
    ///     .try_into()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// For get_account and get_account_transaction request, the above convert will return
    /// result not found error instead of None.
    ///
    /// This function calls to batch_send_with_retry, when you batch submit transactions,
    /// to avoid re-submit errors, it is better to use batch_send_without_retry and handle errors
    /// by yourself.
    pub async fn batch_send(&self, requests: Vec<Request>) -> Result<Vec<JsonRpcResponse>, Error> {
        self.batch_send_with_retry(&requests).await
    }

    pub async fn batch_send_with_retry(
        &self,
        requests: &[Request],
    ) -> Result<Vec<JsonRpcResponse>, Error> {
        let mut retries: u32 = 0;
        loop {
            let ret = self.batch_send_without_retry(requests).await;
            match ret {
                Ok(r) => return Ok(r),
                Err(err) => retries = self.handle_retry_error(retries, err).await?,
            }
        }
    }

    pub async fn batch_send_without_retry(
        &self,
        requests: &[Request],
    ) -> Result<Vec<JsonRpcResponse>, Error> {
        let json_requests: Vec<serde_json::Value> = requests
            .iter()
            .enumerate()
            .map(|(i, r)| r.to_json(i))
            .collect();

        let rpc_resps: Vec<JsonRpcResponse> = self.send_json_request(json![&json_requests]).await?;
        if rpc_resps.len() != requests.len() {
            return Err(Error::unexpected_invalid_batch_response(rpc_resps));
        }

        let mut resp_maps: HashMap<usize, JsonRpcResponse> = HashMap::new();
        for resp in rpc_resps {
            let id = self.validate(&resp, 0, requests.len() - 1)?;
            if resp_maps.contains_key(&id) {
                return Err(Error::unexpected_duplicated_response_id(resp));
            }

            resp_maps.insert(id, resp);
        }

        json_requests
            .iter()
            .enumerate()
            .map(|(i, req)| {
                resp_maps
                    .remove(&i)
                    .ok_or_else(|| Error::unexpected_no_response(req.clone()))
            })
            .collect()
    }

    async fn send_json_request<T: for<'de> serde::Deserialize<'de>>(
        &self,
        req: serde_json::Value,
    ) -> Result<T, Error> {
        let resp = self
            .http_client
            .post(self.server_url.clone())
            .json(&req)
            .send()
            .await
            .map_err(Error::NetworkError)?;
        if resp.status() != 200 {
            return Err(Error::InvalidHTTPStatus(format!("{:#?}", resp)));
        }
        resp.json().await.map_err(Error::InvalidHTTPResponse)
    }

    fn validate(
        &self,
        resp: &JsonRpcResponse,
        min_id: usize,
        max_id: usize,
    ) -> Result<usize, Error> {
        if resp.jsonrpc != "2.0" {
            return Err(Error::InvalidRpcResponse(resp.clone()));
        }

        let id = if let Some(ref id) = resp.id {
            if let Ok(index) = serde_json::from_value::<usize>(id.clone()) {
                if index > max_id || index < min_id {
                    return Err(Error::unexpected_invalid_response_id(resp.clone()));
                }
                index
            } else {
                return Err(Error::unexpected_invalid_response_id_type(resp.clone()));
            }
        } else {
            return Err(Error::unexpected_response_id_not_found(resp.clone()));
        };

        if let Some(state) = self.last_known_state() {
            if resp.diem_chain_id != state.chain_id {
                return Err(Error::ChainIdMismatch(resp.clone()));
            }
        }

        let resp_state = State::from_response(resp);
        if !self.update_state(resp_state) {
            return Err(Error::StaleResponseError(resp.clone()));
        }

        if let Some(ref err) = resp.error {
            return Err(Error::JsonRpcError(err.clone()));
        }
        Ok(id)
    }

    async fn handle_retry_error(&self, mut retries: u32, err: Error) -> Result<u32, Error> {
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
            Ok(retries)
        } else {
            Err(err)
        }
    }
}
