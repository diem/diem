// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::Method;
use crate::{
    views::{
        AccountStateWithProofView, AccountTransactionsWithProofView, AccountView,
        AccumulatorConsistencyProofView, CurrencyInfoView, EventByVersionWithProofView, EventView,
        EventWithProofView, MetadataView, StateProofView, TransactionView,
        TransactionsWithProofsView,
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

    pub fn and_then<U, E, F>(self, f: F) -> Result<Response<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        let (inner, state) = self.into_parts();
        match f(inner) {
            Ok(new_inner) => Ok(Response::new(new_inner, state)),
            Err(err) => Err(err),
        }
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

    //
    // Experimental APIs
    //
    GetStateProof(StateProofView),
    GetAccumulatorConsistencyProof(AccumulatorConsistencyProofView),
    GetAccountStateWithProof(AccountStateWithProofView),
    GetTransactionsWithProofs(Option<TransactionsWithProofsView>),
    GetAccountTransactionsWithProofs(AccountTransactionsWithProofView),
    GetEventsWithProofs(Vec<EventWithProofView>),
    GetEventByVersionWithProof(EventByVersionWithProofView),
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
            Method::GetAccumulatorConsistencyProof => {
                MethodResponse::GetAccumulatorConsistencyProof(serde_json::from_value(json)?)
            }
            Method::GetAccountStateWithProof => {
                MethodResponse::GetAccountStateWithProof(serde_json::from_value(json)?)
            }
            Method::GetTransactionsWithProofs => {
                MethodResponse::GetTransactionsWithProofs(serde_json::from_value(json)?)
            }
            Method::GetAccountTransactionsWithProofs => {
                MethodResponse::GetAccountTransactionsWithProofs(serde_json::from_value(json)?)
            }
            Method::GetEventsWithProofs => {
                MethodResponse::GetEventsWithProofs(serde_json::from_value(json)?)
            }
            Method::GetEventByVersionWithProof => {
                MethodResponse::GetEventByVersionWithProof(serde_json::from_value(json)?)
            }
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
            MethodResponse::GetAccumulatorConsistencyProof(_) => {
                Method::GetAccumulatorConsistencyProof
            }
            MethodResponse::GetAccountStateWithProof(_) => Method::GetAccountStateWithProof,
            MethodResponse::GetTransactionsWithProofs(_) => Method::GetTransactionsWithProofs,
            MethodResponse::GetAccountTransactionsWithProofs(_) => {
                Method::GetAccountTransactionsWithProofs
            }
            MethodResponse::GetEventsWithProofs(_) => Method::GetEventsWithProofs,
            MethodResponse::GetEventByVersionWithProof(_) => Method::GetEventByVersionWithProof,
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

    pub fn try_into_submit(self) -> Result<(), Error> {
        match self {
            MethodResponse::Submit => Ok(()),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::Submit found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_metadata(self) -> Result<MetadataView, Error> {
        match self {
            MethodResponse::GetMetadata(view) => Ok(view),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetMetadata found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_state_proof(self) -> Result<StateProofView, Error> {
        match self {
            MethodResponse::GetStateProof(state_proof) => Ok(state_proof),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetStateProof found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_accumulator_consistency_proof(
        self,
    ) -> Result<AccumulatorConsistencyProofView, Error> {
        match self {
            MethodResponse::GetAccumulatorConsistencyProof(proof) => Ok(proof),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetAccumulatorConsistencyProof found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_account(self) -> Result<Option<AccountView>, Error> {
        match self {
            MethodResponse::GetAccount(account_view) => Ok(account_view),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetAccount found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_transactions(self) -> Result<Vec<TransactionView>, Error> {
        match self {
            MethodResponse::GetTransactions(txs) => Ok(txs),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetTransactions found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_account_transaction(self) -> Result<Option<TransactionView>, Error> {
        match self {
            MethodResponse::GetAccountTransaction(tx) => Ok(tx),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetAccountTransaction found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_account_transactions(self) -> Result<Vec<TransactionView>, Error> {
        match self {
            MethodResponse::GetAccountTransactions(txs) => Ok(txs),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetAccountTransactions found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_account_state_with_proof(self) -> Result<AccountStateWithProofView, Error> {
        match self {
            MethodResponse::GetAccountStateWithProof(view) => Ok(view),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetAccountStateWithProof found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_events(self) -> Result<Vec<EventView>, Error> {
        match self {
            MethodResponse::GetEvents(events) => Ok(events),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetEvents found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_event_by_version_with_proof(
        self,
    ) -> Result<EventByVersionWithProofView, Error> {
        match self {
            MethodResponse::GetEventByVersionWithProof(view) => Ok(view),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetEventByVersionWithProof found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_currencies(self) -> Result<Vec<CurrencyInfoView>, Error> {
        match self {
            MethodResponse::GetCurrencies(currencies) => Ok(currencies),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetCurrencies found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }

    pub fn try_into_get_network_status(self) -> Result<u64, Error> {
        match self {
            MethodResponse::GetNetworkStatus(status) => Ok(status),
            _ => Err(Error::rpc_response(format!(
                "expected MethodResponse::GetNetworkStatus found MethodResponse::{:?}",
                self.method()
            ))),
        }
    }
}
