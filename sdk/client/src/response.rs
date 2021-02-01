// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::Method;
use crate::{
    views::{
        AccountStateWithProofView, AccountView, CurrencyInfoView, EventView, MetadataView,
        StateProofView, TransactionView,
    },
    Error, State,
};
use serde_json::Value;

#[derive(Debug)]
pub struct Response<T> {
    inner: T,

    state: State,
}

impl<T> Response<T> {
    pub fn new(inner: T, state: State) -> Self {
        Self { inner, state }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn into_parts(self) -> (T, State) {
        (self.inner, self.state)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Debug)]
pub enum MethodResponse {
    Submit,
    GetMetadata(MetadataView),
    GetAccount(Option<AccountView>),
    GetTransactions(Vec<TransactionView>),
    GetAccountTransaction(Option<TransactionView>),
    GetAccountTransactions(Vec<TransactionView>),
    GetEvents(Vec<EventView>),
    GetCurrencies(Vec<CurrencyInfoView>),
    GetNetworkStatus(u64),

    GetStateProof(StateProofView),
    GetAccountStateWithProof(AccountStateWithProofView),
    GetTransactionsWithProofs,
    GetEventsWithProofs,
}

impl MethodResponse {
    pub fn from_json(method: Method, json: Value) -> Result<Self, Error> {
        let response = match method {
            Method::Submit => MethodResponse::Submit,
            Method::GetMetadata => MethodResponse::GetMetadata(serde_json::from_value(json)?),
            Method::GetAccount => MethodResponse::GetAccount(serde_json::from_value(json)?),
            Method::GetTransactions => {
                MethodResponse::GetTransactions(serde_json::from_value(json)?)
            }
            Method::GetAccountTransaction => {
                MethodResponse::GetAccountTransaction(serde_json::from_value(json)?)
            }
            Method::GetAccountTransactions => {
                MethodResponse::GetAccountTransactions(serde_json::from_value(json)?)
            }
            Method::GetEvents => MethodResponse::GetEvents(serde_json::from_value(json)?),
            Method::GetCurrencies => MethodResponse::GetCurrencies(serde_json::from_value(json)?),
            Method::GetNetworkStatus => {
                MethodResponse::GetNetworkStatus(serde_json::from_value(json)?)
            }
            Method::GetStateProof => MethodResponse::GetStateProof(serde_json::from_value(json)?),
            Method::GetAccountStateWithProof => {
                MethodResponse::GetAccountStateWithProof(serde_json::from_value(json)?)
            }
            Method::GetTransactionsWithProofs => MethodResponse::GetTransactionsWithProofs,
            Method::GetEventsWithProofs => MethodResponse::GetEventsWithProofs,
        };

        Ok(response)
    }

    pub fn method(&self) -> Method {
        match self {
            MethodResponse::Submit => Method::Submit,
            MethodResponse::GetMetadata(_) => Method::GetMetadata,
            MethodResponse::GetAccount(_) => Method::GetAccount,
            MethodResponse::GetTransactions(_) => Method::GetTransactions,
            MethodResponse::GetAccountTransaction(_) => Method::GetAccountTransaction,
            MethodResponse::GetAccountTransactions(_) => Method::GetAccountTransactions,
            MethodResponse::GetEvents(_) => Method::GetEvents,
            MethodResponse::GetCurrencies(_) => Method::GetCurrencies,
            MethodResponse::GetNetworkStatus(_) => Method::GetNetworkStatus,
            MethodResponse::GetStateProof(_) => Method::GetStateProof,
            MethodResponse::GetAccountStateWithProof(_) => Method::GetAccountStateWithProof,
            MethodResponse::GetTransactionsWithProofs => Method::GetTransactionsWithProofs,
            MethodResponse::GetEventsWithProofs => Method::GetEventsWithProofs,
        }
    }

    pub fn unwrap_get_account(self) -> Option<AccountView> {
        if let MethodResponse::GetAccount(inner) = self {
            inner
        } else {
            panic!(
                "expected MethodResponse::GetAccount found MethodResponse::{:?}",
                self.method()
            );
        }
    }
}
