// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::{
    defaults, types as jsonrpc, BroadcastHttpClient, Error, HttpClient, JsonRpcResponse, Request,
    Response, RetryStrategy, SimpleHttpClient, State, WaitForTransactionError,
};
use diem_crypto::hash::CryptoHash;
use diem_types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, Transaction},
};

use serde::de::DeserializeOwned;

use std::{convert::TryInto, sync::Arc, time::Duration};

pub struct Client<R> {
    pub http_client: Arc<dyn HttpClient>,
    pub retry: R,
}

impl<R: RetryStrategy> Client<R> {
    pub fn from_url<T: reqwest::IntoUrl>(server_url: T, retry: R) -> Result<Self, reqwest::Error> {
        Ok(Self {
            http_client: Arc::new(SimpleHttpClient::new(server_url)?),
            retry,
        })
    }

    pub fn from_url_list<T: reqwest::IntoUrl>(
        server_urls: Vec<T>,
        num_parallel_requests: usize,
        retry: R,
    ) -> Result<Self, String> {
        Ok(Self {
            http_client: Arc::new(BroadcastHttpClient::new(
                server_urls,
                num_parallel_requests,
            )?),
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
            if let Some(state) = self.http_client.last_known_state() {
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
        self.http_client.last_known_state()
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
        let req = Request::submit(txn).map_err(Error::unexpected_bcs_error)?;
        let resp = self.http_client.single_request(&req).await?;
        Ok(Response {
            result: (),
            state: State::from_response(&resp),
        })
    }

    pub async fn send<T: DeserializeOwned>(&self, request: Request) -> Result<Response<T>, Error> {
        self.send_with_retry(&request, &self.retry)
            .await?
            .try_into()
    }

    pub async fn send_opt<T: DeserializeOwned>(
        &self,
        request: Request,
    ) -> Result<Response<Option<T>>, Error> {
        let resp = self.send_with_retry(&request, &self.retry).await?;
        let state = State::from_response(&resp);
        let result = match resp.result {
            Some(ret) => {
                Some(serde_json::from_value(ret).map_err(Error::DeserializeResponseJsonError)?)
            }
            None => None,
        };
        Ok(Response { result, state })
    }

    pub async fn send_with_retry<RS: RetryStrategy>(
        &self,
        request: &Request,
        retry: &RS,
    ) -> Result<JsonRpcResponse, Error> {
        let mut retries: u32 = 0;
        loop {
            let ret = self.http_client.single_request(request).await;
            match ret {
                Ok(r) => return Ok(r),
                Err(err) => retries = self.handle_retry_error(retries, err, retry).await?,
            }
        }
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
        self.batch_send_with_retry(&requests, &self.retry).await
    }

    pub async fn batch_send_with_retry<RS: RetryStrategy>(
        &self,
        requests: &[Request],
        retry: &RS,
    ) -> Result<Vec<JsonRpcResponse>, Error> {
        let mut retries: u32 = 0;
        loop {
            let ret = self.http_client.batch_request(requests).await;
            match ret {
                Ok(r) => return Ok(r),
                Err(err) => retries = self.handle_retry_error(retries, err, retry).await?,
            }
        }
    }

    async fn handle_retry_error<RS: RetryStrategy>(
        &self,
        mut retries: u32,
        err: Error,
        retry: &RS,
    ) -> Result<u32, Error> {
        if !retry.is_retriable(&err) {
            return Err(err);
        }
        if retries < retry.max_retries(&err) {
            match retries.checked_add(1) {
                Some(i) if i < retry.max_retries(&err) => {
                    retries = i;
                }
                _ => return Err(err),
            };
            tokio::time::delay_for(retry.delay(&err, retries)).await;
            Ok(retries)
        } else {
            Err(err)
        }
    }
}
