// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    request::{JsonRpcRequest, MethodRequest},
    response::{MethodResponse, Response},
    state::StateManager,
    validate, validate_batch, BatchResponse, USER_AGENT,
};
use crate::{
    error::WaitForTransactionError,
    move_deserialize::{self, Event},
    views::{
        AccountStateWithProofView, AccountTransactionsWithProofView, AccountView,
        AccumulatorConsistencyProofView, CurrencyInfoView, EventByVersionWithProofView, EventView,
        EventWithProofView, MetadataView, StateProofView, TransactionView,
        TransactionsWithProofsView,
    },
    Error, Result, Retry, State,
};
use diem_crypto::{hash::CryptoHash, HashValue};
use diem_types::{
    account_address::AccountAddress,
    event::EventKey,
    transaction::{SignedTransaction, Transaction},
};
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

// In order to avoid needing to publish the proxy crate to crates.io we simply include the small
// library in inline by making it a module instead of a dependency. 'src/proxy.rs' is a symlink to
// '../../../common/proxy/src/lib.rs'
#[path = "proxy.rs"]
mod proxy;

const REQUEST_TIMEOUT: u64 = 10_000;

#[derive(Clone, Debug)]
pub struct BlockingClient {
    url: String,
    state: StateManager,
    retry: Retry,
}

impl BlockingClient {
    pub fn new<T: Into<String>>(url: T) -> Self {
        Self {
            url: url.into(),
            state: StateManager::new(),
            retry: Retry::default(),
        }
    }

    pub fn last_known_state(&self) -> Option<State> {
        self.state.last_known_state()
    }

    pub fn wait_for_signed_transaction(
        &self,
        txn: &SignedTransaction,
        timeout: Option<Duration>,
        delay: Option<Duration>,
    ) -> Result<Response<TransactionView>, WaitForTransactionError> {
        let response = self.wait_for_transaction(
            txn.sender(),
            txn.sequence_number(),
            txn.expiration_timestamp_secs(),
            Transaction::UserTransaction(txn.clone()).hash(),
            timeout,
            delay,
        )?;

        if !response.inner().vm_status.is_executed() {
            return Err(WaitForTransactionError::TransactionExecutionFailed(
                response.into_inner(),
            ));
        }

        Ok(response)
    }

    pub fn wait_for_transaction(
        &self,
        address: AccountAddress,
        seq: u64,
        expiration_time_secs: u64,
        txn_hash: HashValue,
        timeout: Option<Duration>,
        delay: Option<Duration>,
    ) -> Result<Response<TransactionView>, WaitForTransactionError> {
        const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
        const DEFAULT_DELAY: Duration = Duration::from_millis(500);

        let start = std::time::Instant::now();
        while start.elapsed() < timeout.unwrap_or(DEFAULT_TIMEOUT) {
            let txn_resp = self
                .get_account_transaction(address, seq, true)
                .map_err(WaitForTransactionError::GetTransactionError)?;
            if let (Some(txn), state) = txn_resp.into_parts() {
                if txn.hash != txn_hash {
                    return Err(WaitForTransactionError::TransactionHashMismatchError(txn));
                }

                return Ok(Response::new(txn, state));
            }

            if let Some(state) = self.last_known_state() {
                if expiration_time_secs <= state.timestamp_usecs / 1_000_000 {
                    return Err(WaitForTransactionError::TransactionExpired);
                }
            }
            std::thread::sleep(delay.unwrap_or(DEFAULT_DELAY));
        }

        Err(WaitForTransactionError::Timeout)
    }

    pub fn batch(
        &self,
        requests: Vec<MethodRequest>,
    ) -> Result<Vec<Result<Response<MethodResponse>>>> {
        self.send_batch(requests)
    }

    pub fn request(&self, request: MethodRequest) -> Result<Response<MethodResponse>> {
        let method = request.method();
        let resp: Response<serde_json::Value> = self.send(request)?;
        resp.and_then(|json| MethodResponse::from_json(method, json).map_err(Error::decode))
    }

    pub fn submit(&self, txn: &SignedTransaction) -> Result<Response<()>> {
        let request = JsonRpcRequest::new(MethodRequest::submit(txn).map_err(Error::request)?);
        self.send_without_retry(&request, true)
    }

    pub fn get_metadata_by_version(&self, version: u64) -> Result<Response<MetadataView>> {
        self.send(MethodRequest::get_metadata_by_version(version))
    }

    pub fn get_metadata(&self) -> Result<Response<MetadataView>> {
        self.send(MethodRequest::get_metadata())
    }

    pub fn get_account(&self, address: AccountAddress) -> Result<Response<Option<AccountView>>> {
        self.send(MethodRequest::get_account(address))
    }

    pub fn get_account_by_version(
        &self,
        address: AccountAddress,
        version: u64,
    ) -> Result<Response<Option<AccountView>>> {
        self.send(MethodRequest::get_account_by_version(address, version))
    }

    pub fn get_transactions(
        &self,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Vec<TransactionView>>> {
        self.send(MethodRequest::get_transactions(
            start_seq,
            limit,
            include_events,
        ))
    }

    pub fn get_account_transaction(
        &self,
        address: AccountAddress,
        seq: u64,
        include_events: bool,
    ) -> Result<Response<Option<TransactionView>>> {
        self.send(MethodRequest::get_account_transaction(
            address,
            seq,
            include_events,
        ))
    }

    pub fn get_account_transactions(
        &self,
        address: AccountAddress,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Vec<TransactionView>>> {
        self.send(MethodRequest::get_account_transactions(
            address,
            start_seq,
            limit,
            include_events,
        ))
    }

    pub fn get_events(
        &self,
        key: EventKey,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<EventView>>> {
        self.send(MethodRequest::get_events(key, start_seq, limit))
    }

    pub fn get_currencies(&self) -> Result<Response<Vec<CurrencyInfoView>>> {
        self.send(MethodRequest::get_currencies())
    }

    pub fn get_network_status(&self) -> Result<Response<u64>> {
        self.send(MethodRequest::get_network_status())
    }

    //
    // Experimental APIs
    //

    pub fn get_state_proof(&self, from_version: u64) -> Result<Response<StateProofView>> {
        self.send(MethodRequest::get_state_proof(from_version))
    }

    pub fn get_accumulator_consistency_proof(
        &self,
        client_known_version: Option<u64>,
        ledger_version: Option<u64>,
    ) -> Result<Response<AccumulatorConsistencyProofView>> {
        self.send(MethodRequest::get_accumulator_consistency_proof(
            client_known_version,
            ledger_version,
        ))
    }

    pub fn get_account_state_with_proof(
        &self,
        address: AccountAddress,
        from_version: Option<u64>,
        to_version: Option<u64>,
    ) -> Result<Response<AccountStateWithProofView>> {
        self.send(MethodRequest::get_account_state_with_proof(
            address,
            from_version,
            to_version,
        ))
    }

    pub fn get_transactions_with_proofs(
        &self,
        start_version: u64,
        limit: u64,
        include_events: bool,
    ) -> Result<Response<Option<TransactionsWithProofsView>>> {
        self.send(MethodRequest::get_transactions_with_proofs(
            start_version,
            limit,
            include_events,
        ))
    }

    pub fn get_account_transactions_with_proofs(
        &self,
        address: AccountAddress,
        start_seq: u64,
        limit: u64,
        include_events: bool,
        ledger_version: Option<u64>,
    ) -> Result<Response<AccountTransactionsWithProofView>> {
        self.send(MethodRequest::get_account_transactions_with_proofs(
            address,
            start_seq,
            limit,
            include_events,
            ledger_version,
        ))
    }

    pub fn get_events_with_proofs(
        &self,
        key: EventKey,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<EventWithProofView>>> {
        self.send(MethodRequest::get_events_with_proofs(key, start_seq, limit))
    }

    pub fn get_event_by_version_with_proof(
        &self,
        key: EventKey,
        version: Option<u64>,
    ) -> Result<Response<EventByVersionWithProofView>> {
        self.send(MethodRequest::get_event_by_version_with_proof(key, version))
    }

    /// Return the events of type `T` that have been emitted to `event_key` since `start_seq`, with a max of `limit`
    /// results
    /// Returns an empty vector if there are no such event
    /// The type `T` must match the event types associated with `event_key`
    pub fn get_deserialized_events<T: MoveStructType + DeserializeOwned>(
        &self,
        event_key: &EventKey,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<Event<T>>>> {
        let (events, state) = self
            .get_events_with_proofs(*event_key, start_seq, limit)?
            .into_parts();
        Ok(Response::new(
            move_deserialize::get_events::<T>(events)?,
            state,
        ))
    }

    /// Deserialize and return the resource value of type `T` stored under `address`
    /// Returns None if there is no such value
    pub fn get_deserialized_resource<T: MoveResource>(
        &self,
        address: AccountAddress,
    ) -> Result<Response<Option<T>>> {
        let (account, state) = self
            .get_account_state_with_proof(address, None, None)?
            .into_parts();
        Ok(Response::new(
            move_deserialize::get_resource(account)?,
            state,
        ))
    }

    //
    // Private Helpers
    //

    fn send<T: DeserializeOwned>(&self, request: MethodRequest) -> Result<Response<T>> {
        let request = JsonRpcRequest::new(request);
        self.retry
            .retry(|| self.send_without_retry(&request, false))
    }

    fn send_without_retry<T: DeserializeOwned>(
        &self,
        request: &JsonRpcRequest,
        ignore_stale: bool,
    ) -> Result<Response<T>> {
        let req_state = self.last_known_state();
        let resp: diem_json_rpc_types::response::JsonRpcResponse = self.send_impl(&request)?;

        let (id, state, result) = validate(&self.state, req_state.as_ref(), &resp, ignore_stale)?;

        if request.id() != id {
            return Err(Error::rpc_response("invalid response id"));
        }

        let inner = serde_json::from_value(result).map_err(Error::decode)?;
        Ok(Response::new(inner, state))
    }

    fn send_batch(
        &self,
        requests: Vec<MethodRequest>,
    ) -> Result<Vec<Result<Response<MethodResponse>>>> {
        let request: Vec<JsonRpcRequest> = requests.into_iter().map(JsonRpcRequest::new).collect();
        let req_state = self.last_known_state();
        let resp: BatchResponse = self.send_impl(&request)?;

        let resp = resp.success()?;

        validate_batch(&self.state, req_state.as_ref(), &request, resp)
    }

    // Executes the specified request method using the given parameters by contacting the JSON RPC
    // server. If the 'http_proxy' or 'https_proxy' environment variable is set, enable the proxy.
    fn send_impl<S: Serialize, T: DeserializeOwned>(&self, payload: &S) -> Result<T> {
        let mut request = ureq::post(&self.url)
            .timeout_connect(REQUEST_TIMEOUT)
            .set("User-Agent", USER_AGENT)
            .build();

        let proxy = proxy::Proxy::new();
        let host = request.get_host().expect("unable to get the host");
        let scheme = request
            .get_scheme()
            .expect("Unable to get the scheme from the host");
        let proxy_url = match scheme.as_str() {
            "http" => proxy.http(&host),
            "https" => proxy.https(&host),
            _ => None,
        };
        if let Some(proxy_url) = proxy_url {
            request.set_proxy(ureq::Proxy::new(proxy_url).expect("Unable to parse proxy_url"));
        }

        let resp = request.send_json(serde_json::json!(payload));

        if resp.synthetic() {
            let e = resp.into_synthetic_error().unwrap();
            let error = match &e {
                ureq::Error::BadUrl(_)
                | ureq::Error::UnknownScheme(_)
                | ureq::Error::DnsFailed(_)
                | ureq::Error::BadHeader
                | ureq::Error::BadProxy
                | ureq::Error::BadProxyCreds
                | ureq::Error::ProxyConnect
                | ureq::Error::InvalidProxyCreds
                | ureq::Error::ConnectionFailed(_)
                | ureq::Error::TlsError(_) => Error::request(e),
                ureq::Error::Io(io_error) => {
                    if let std::io::ErrorKind::TimedOut = io_error.kind() {
                        Error::timeout(e)
                    } else {
                        Error::unknown(e)
                    }
                }
                _ => Error::unknown(e),
            };

            return Err(error);
        }

        if resp.status() != 200 {
            return Err(Error::status(resp.status()));
        }

        resp.into_json_deserialize().map_err(Error::decode)
    }
}
