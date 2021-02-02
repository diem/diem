// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{
    request::{JsonRpcRequest, MethodRequest},
    response::{MethodResponse, Response},
    validate, validate_batch, BatchResponse,
};
use crate::{
    views::{
        AccountStateWithProofView, AccountView, CurrencyInfoView, EventView, MetadataView,
        StateProofView, TransactionView,
    },
    Error, Result,
};
use diem_types::{account_address::AccountAddress, transaction::SignedTransaction};
use reqwest::Client as ReqwestClient;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Client {
    url: String,
    inner: ReqwestClient,
}

impl Client {
    pub fn new<T: Into<String>>(url: T) -> Self {
        let inner = ReqwestClient::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        Self {
            url: url.into(),
            inner,
        }
    }

    pub async fn batch(
        &self,
        requests: Vec<MethodRequest>,
    ) -> Result<Vec<Result<Response<MethodResponse>>>> {
        self.send_batch(requests).await
    }

    pub async fn submit(&self, txn: &SignedTransaction) -> Result<Response<()>> {
        self.send(MethodRequest::submit(txn).map_err(Error::request)?)
            .await
    }

    pub async fn get_metadata_by_version(&self, version: u64) -> Result<Response<MetadataView>> {
        self.send(MethodRequest::get_metadata_by_version(version))
            .await
    }

    pub async fn get_metadata(&self) -> Result<Response<MetadataView>> {
        self.send(MethodRequest::get_metadata()).await
    }

    pub async fn get_account(
        &self,
        address: AccountAddress,
    ) -> Result<Response<Option<AccountView>>> {
        self.send(MethodRequest::get_account(address)).await
    }

    pub async fn get_transactions(
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
        .await
    }

    pub async fn get_account_transaction(
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
        .await
    }

    pub async fn get_account_transactions(
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
        .await
    }

    pub async fn get_events(
        &self,
        key: &str,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<EventView>>> {
        self.send(MethodRequest::get_events(key, start_seq, limit))
            .await
    }

    pub async fn get_currencies(&self) -> Result<Response<Vec<CurrencyInfoView>>> {
        self.send(MethodRequest::get_currencies()).await
    }

    pub async fn get_network_status(&self) -> Result<Response<u64>> {
        self.send(MethodRequest::get_network_status()).await
    }

    //
    // Experimental APIs
    //

    pub async fn get_state_proof(&self, from_version: u64) -> Result<Response<StateProofView>> {
        self.send(MethodRequest::get_state_proof(from_version))
            .await
    }

    pub async fn get_account_state_with_proof(
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
        .await
    }

    pub async fn get_transactions_with_proofs(
        &self,
        start_version: u64,
        limit: u64,
    ) -> Result<Response<()>> {
        self.send(MethodRequest::get_transactions_with_proofs(
            start_version,
            limit,
        ))
        .await
    }

    pub async fn get_events_with_proofs(
        &self,
        key: &str,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<()>> {
        self.send(MethodRequest::get_events_with_proofs(key, start_seq, limit))
            .await
    }

    //
    // Private Helpers
    //

    async fn send<T: DeserializeOwned>(&self, request: MethodRequest) -> Result<Response<T>> {
        let request = JsonRpcRequest::new(request);
        let resp: diem_json_rpc_types::response::JsonRpcResponse = self.send_impl(&request).await?;

        let (id, state, result) = validate(&resp)?;

        if request.id() != id {
            return Err(Error::rpc_response("invalid response id"));
        }

        let inner = serde_json::from_value(result).map_err(Error::decode)?;
        Ok(Response::new(inner, state))
    }

    async fn send_batch(
        &self,
        requests: Vec<MethodRequest>,
    ) -> Result<Vec<Result<Response<MethodResponse>>>> {
        let request: Vec<JsonRpcRequest> = requests.into_iter().map(JsonRpcRequest::new).collect();
        let resp: BatchResponse = self.send_impl(&request).await?;

        let resp = resp.success()?;

        validate_batch(&request, resp)
    }

    async fn send_impl<S: Serialize, T: DeserializeOwned>(&self, payload: &S) -> Result<T> {
        let response = self
            .inner
            .post(&self.url)
            .json(payload)
            .send()
            .await
            .map_err(Error::from_reqwest_error)?;

        if response.status() != 200 {
            return Err(Error::status(response.status().as_u16()));
        }

        response.json().await.map_err(Error::from_reqwest_error)
    }
}
