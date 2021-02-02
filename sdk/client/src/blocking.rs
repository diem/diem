// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    request::{JsonRpcRequest, MethodRequest},
    response::{MethodResponse, Response},
    state::StateManager,
    validate, validate_batch, BatchResponse,
};
use crate::{
    views::{
        AccountStateWithProofView, AccountView, CurrencyInfoView, EventView, MetadataView,
        StateProofView, TransactionView,
    },
    Error, Result, State,
};
use diem_types::{account_address::AccountAddress, transaction::SignedTransaction};
use serde::{de::DeserializeOwned, Serialize};

const REQUEST_TIMEOUT: u64 = 10_000;

#[derive(Clone, Debug)]
pub struct BlockingClient {
    url: String,
    state: StateManager,
}

impl BlockingClient {
    pub fn new<T: Into<String>>(url: T) -> Self {
        Self {
            url: url.into(),
            state: StateManager::new(),
        }
    }

    pub fn last_known_state(&self) -> Option<State> {
        self.state.last_known_state()
    }

    pub fn batch(
        &self,
        requests: Vec<MethodRequest>,
    ) -> Result<Vec<Result<Response<MethodResponse>>>> {
        self.send_batch(requests)
    }

    pub fn submit(&self, txn: &SignedTransaction) -> Result<Response<()>> {
        self.send(MethodRequest::submit(txn).map_err(Error::request)?)
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
        key: &str,
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
    ) -> Result<Response<()>> {
        self.send(MethodRequest::get_transactions_with_proofs(
            start_version,
            limit,
        ))
    }

    pub fn get_events_with_proofs(
        &self,
        key: &str,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<()>> {
        self.send(MethodRequest::get_events_with_proofs(key, start_seq, limit))
    }

    //
    // Private Helpers
    //

    fn send<T: DeserializeOwned>(&self, request: MethodRequest) -> Result<Response<T>> {
        let request = JsonRpcRequest::new(request);
        let resp: diem_json_rpc_types::response::JsonRpcResponse = self.send_impl(&request)?;

        let (id, state, result) = validate(&self.state, &resp)?;

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
        let resp: BatchResponse = self.send_impl(&request)?;

        let resp = resp.success()?;

        validate_batch(&self.state, &request, resp)
    }

    // Executes the specified request method using the given parameters by contacting the JSON RPC
    // server. If the 'http_proxy' or 'https_proxy' environment variable is set, enable the proxy.
    fn send_impl<S: Serialize, T: DeserializeOwned>(&self, payload: &S) -> Result<T> {
        let mut request = ureq::post(&self.url)
            .timeout_connect(REQUEST_TIMEOUT)
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
