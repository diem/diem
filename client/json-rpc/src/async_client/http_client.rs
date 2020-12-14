// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::{defaults, Error, JsonRpcResponse, State, StateManager};
use async_trait::async_trait;
use futures::future::join_all;

use diem_types::{account_address::AccountAddress, transaction::SignedTransaction};
use rand::seq::SliceRandom;
use serde_json::json;
use std::{collections::HashMap, convert::TryFrom};
use tokio::time::timeout;

#[derive(Debug)]
pub struct Request {
    pub method: &'static str,
    pub params: serde_json::Value,
}

impl Request {
    pub fn new(method: &'static str, params: serde_json::Value) -> Self {
        Request { method, params }
    }

    pub fn submit(txn: &SignedTransaction) -> Result<Self, bcs::Error> {
        let txn_payload = hex::encode(bcs::to_bytes(txn)?);
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

async fn send_json_request<T: for<'de> serde::Deserialize<'de>>(
    client: reqwest::Client,
    url: reqwest::Url,
    req: serde_json::Value,
) -> Result<T, Error> {
    let resp = client
        .post(url)
        .json(&req)
        .send()
        .await
        .map_err(Error::NetworkError)?;
    if !resp.status().is_success() {
        return Err(Error::InvalidHTTPStatus(
            format!("{:#?}", resp),
            resp.status(),
        ));
    }
    resp.json().await.map_err(Error::InvalidHTTPResponse)
}

#[async_trait]
pub trait HttpClient: Sync + Send + 'static {
    async fn single_request(&self, request: &Request) -> Result<JsonRpcResponse, Error>;

    async fn batch_request(&self, requests: &[Request]) -> Result<Vec<JsonRpcResponse>, Error>;

    fn update_state(&self, resp_state: State) -> bool;

    fn last_known_state(&self) -> Option<State>;

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
}

pub struct SimpleHttpClient {
    pub http_client: reqwest::Client,
    pub url: reqwest::Url,
    pub state_manager: StateManager,
}

impl SimpleHttpClient {
    pub fn new<T: reqwest::IntoUrl>(server_url: T) -> Result<Self, reqwest::Error> {
        let reqwest_client = reqwest::ClientBuilder::new()
            .use_native_tls()
            .timeout(defaults::HTTP_REQUEST_TIMEOUT)
            .build()?;
        Ok(Self {
            http_client: reqwest_client,
            url: server_url
                .into_url()
                .expect("Invalid server_url provided to SimpleHttpClient"),
            state_manager: StateManager::new(),
        })
    }
}

#[async_trait]
impl HttpClient for SimpleHttpClient {
    async fn single_request(&self, request: &Request) -> Result<JsonRpcResponse, Error> {
        let id = 1;
        let rpc_resp: JsonRpcResponse = send_json_request(
            self.http_client.clone(),
            self.url.clone(),
            request.to_json(id),
        )
        .await?;
        self.validate(&rpc_resp, id, id)?;
        Ok(rpc_resp)
    }

    async fn batch_request(&self, requests: &[Request]) -> Result<Vec<JsonRpcResponse>, Error> {
        let json_requests: Vec<serde_json::Value> = requests
            .iter()
            .enumerate()
            .map(|(i, r)| r.to_json(i))
            .collect();

        let rpc_resps: Vec<JsonRpcResponse> = send_json_request(
            self.http_client.clone(),
            self.url.clone(),
            json![&json_requests],
        )
        .await?;
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

    fn update_state(&self, resp_state: State) -> bool {
        self.state_manager.update_state(resp_state)
    }

    fn last_known_state(&self) -> Option<State> {
        self.state_manager.last_known_state()
    }
}

pub struct BroadcastHttpClient {
    pub http_client: reqwest::Client,
    pub urls: Vec<reqwest::Url>,
    pub num_parallel_requests: usize,
    pub state_manager: StateManager,
}

impl BroadcastHttpClient {
    pub fn new<T: reqwest::IntoUrl>(
        server_urls: Vec<T>,
        num_parallel_requests: usize,
    ) -> Result<Self, String> {
        if num_parallel_requests < 1 {
            return Err("num_parallel_requests should be >= 1".into());
        }
        if num_parallel_requests > server_urls.len() {
            return Err(format!(
                "num_parallel_requests({}) should be <= length of server_urls({})",
                num_parallel_requests,
                server_urls.len()
            ));
        }
        let server_urls = server_urls
            .into_iter()
            .map(|x| x.into_url())
            .collect::<Result<Vec<_>, reqwest::Error>>()
            .map_err(|x| format!("{}", x))?;

        let reqwest_client = reqwest::ClientBuilder::new()
            .use_native_tls()
            .timeout(defaults::HTTP_REQUEST_TIMEOUT)
            .build()
            .map_err(|err| format!("{}", err))?;
        Ok(Self {
            http_client: reqwest_client,
            urls: server_urls,
            num_parallel_requests,
            state_manager: StateManager::new(),
        })
    }
    async fn multi_send_json_request<T: for<'de> serde::Deserialize<'de>>(
        &self,
        req: serde_json::Value,
    ) -> Result<Vec<T>, Error> {
        // Choose `num_parallel_requests` from `self.server_urls` to send requests to
        let chosen_urls: Vec<reqwest::Url> = self
            .urls
            .choose_multiple(&mut rand::thread_rng(), self.num_parallel_requests)
            .cloned()
            .collect();
        let futures = chosen_urls.iter().map(|url| {
            let url = url.clone();
            let req = req.clone();
            let client = self.http_client.clone();
            async move {
                match timeout(
                    defaults::TIMEOUT,
                    send_json_request(client, url.clone(), req),
                )
                .await
                {
                    Err(_) => Err(Error::ResponseTimeout(format!(
                        "JSON RPC Timed out waiting for response from server {}",
                        url
                    ))),
                    Ok(result) => result,
                }
            }
        });
        let mut results: Vec<Result<T, Error>> = join_all(futures).await.into_iter().collect();
        if results.is_empty() {
            return Err(Error::unexpected_uncategorized(
                "Expected atleast one response in multi_send_json_request. Found none".into(),
            ));
        }
        // All of them returned an error, so return the first error
        if results.iter().all(|x| x.is_err()) {
            let first_element = results.swap_remove(0);
            if let Err(e) = first_element {
                return Err(e);
            }
        }
        Ok(results.into_iter().filter_map(|x| x.ok()).collect())
    }

    /// Select the JsonRpcResponse which has the the highest diem ledger version
    pub fn select_highest_ledger_version_response(
        &self,
        id: usize,
        rpc_resp: Vec<JsonRpcResponse>,
    ) -> Result<JsonRpcResponse, Error> {
        let mut rpc_resp: Vec<Result<JsonRpcResponse, Error>> = rpc_resp
            .into_iter()
            .map(|json_rpc_response| {
                if let Err(err) = self.validate(&json_rpc_response, id, id) {
                    Err(err)
                } else {
                    Ok(json_rpc_response)
                }
            })
            .collect();
        // If all servers return an Error, return the first Error back
        if rpc_resp.iter().all(|x| x.is_err()) {
            return rpc_resp.swap_remove(0);
        }
        // Filter out all the errors and choose the JsonRpcResponse with the highest diem_ledger_version and return that
        let mut rpc_resp: Vec<JsonRpcResponse> =
            rpc_resp.into_iter().filter_map(|x| x.ok()).collect();
        rpc_resp.sort_by(|x, y| y.diem_ledger_version.cmp(&x.diem_ledger_version));
        Ok(rpc_resp.swap_remove(0))
    }

    /// Select the Vec<JsonRpcResponse> which has the the highest diem ledger version
    pub fn select_highest_ledger_version_response_batch(
        &self,
        json_requests: Vec<serde_json::Value>,
        rpc_resps_list: Vec<Vec<JsonRpcResponse>>,
    ) -> Result<Vec<JsonRpcResponse>, Error> {
        let mut rpc_resps_list = rpc_resps_list
            .into_iter()
            .map(|rpc_resps| {
                if rpc_resps.len() != json_requests.len() {
                    return Err(Error::unexpected_invalid_batch_response(rpc_resps));
                }
                let mut resp_maps: HashMap<usize, JsonRpcResponse> = HashMap::new();
                for resp in rpc_resps {
                    let id = self.validate(&resp, 0, json_requests.len() - 1)?;
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
                    .collect::<Result<Vec<JsonRpcResponse>, Error>>()
            })
            .collect::<Vec<_>>();
        // If all servers return an Error, return the first Error back
        if rpc_resps_list.iter().all(|x| x.is_err()) {
            return rpc_resps_list.swap_remove(0);
        }
        // Filter out all the errors and choose the JsonRpcResponse with the highest diem_ledger_version and return that
        let mut rpc_resps_list = rpc_resps_list
            .into_iter()
            .filter_map(|x| x.ok())
            .collect::<Vec<_>>();
        rpc_resps_list.sort_by(|x, y| {
            let max_y_diem_ledger_version = y.iter().map(|z| z.diem_ledger_version).max().unwrap();
            let max_x_diem_ledger_version = x.iter().map(|z| z.diem_ledger_version).max().unwrap();
            max_y_diem_ledger_version.cmp(&max_x_diem_ledger_version)
        });
        Ok(rpc_resps_list.swap_remove(0))
    }
}
#[async_trait]
impl HttpClient for BroadcastHttpClient {
    async fn single_request(&self, request: &Request) -> Result<JsonRpcResponse, Error> {
        let id = 1;
        let rpc_resp: Vec<JsonRpcResponse> =
            self.multi_send_json_request(request.to_json(id)).await?;
        self.select_highest_ledger_version_response(id, rpc_resp)
    }

    async fn batch_request(&self, requests: &[Request]) -> Result<Vec<JsonRpcResponse>, Error> {
        let json_requests: Vec<serde_json::Value> = requests
            .iter()
            .enumerate()
            .map(|(i, r)| r.to_json(i))
            .collect();
        let rpc_resps_list: Vec<Vec<JsonRpcResponse>> =
            self.multi_send_json_request(json![&json_requests]).await?;
        self.select_highest_ledger_version_response_batch(json_requests, rpc_resps_list)
    }

    fn update_state(&self, resp_state: State) -> bool {
        self.state_manager.update_state(resp_state)
    }

    fn last_known_state(&self) -> Option<State> {
        self.state_manager.last_known_state()
    }
}
