// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{JsonRpcVersion, Method};
use diem_types::{
    account_address::AccountAddress, event::EventKey, transaction::SignedTransaction,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum MethodRequest {
    Submit((String,)),
    GetMetadata((Option<u64>,)),
    GetAccount(AccountAddress, Option<u64>),
    GetTransactions(u64, u64, bool),
    GetAccountTransaction(AccountAddress, u64, bool),
    GetAccountTransactions(AccountAddress, u64, u64, bool),
    GetEvents(EventKey, u64, u64),
    GetCurrencies([(); 0]),
    GetNetworkStatus([(); 0]),

    //
    // Experimental APIs
    //
    GetStateProof((u64,)),
    GetAccumulatorConsistencyProof(Option<u64>, Option<u64>),
    GetAccountStateWithProof(AccountAddress, Option<u64>, Option<u64>),
    GetTransactionsWithProofs(u64, u64, bool),
    GetAccountTransactionsWithProofs(AccountAddress, u64, u64, bool, Option<u64>),
    GetEventsWithProofs(EventKey, u64, u64),
    GetEventByVersionWithProof(EventKey, Option<u64>),
}

impl MethodRequest {
    pub fn submit(txn: &SignedTransaction) -> Result<Self, bcs::Error> {
        let txn_payload = hex::encode(bcs::to_bytes(txn)?);
        Ok(Self::Submit((txn_payload,)))
    }

    pub fn get_metadata_by_version(version: u64) -> Self {
        Self::GetMetadata((Some(version),))
    }

    pub fn get_metadata() -> Self {
        Self::GetMetadata((None,))
    }

    pub fn get_account_by_version(address: AccountAddress, version: u64) -> Self {
        Self::GetAccount(address, Some(version))
    }

    pub fn get_account(address: AccountAddress) -> Self {
        Self::GetAccount(address, None)
    }

    pub fn get_transactions(start_seq: u64, limit: u64, include_events: bool) -> Self {
        Self::GetTransactions(start_seq, limit, include_events)
    }

    pub fn get_account_transaction(
        address: AccountAddress,
        seq: u64,
        include_events: bool,
    ) -> Self {
        Self::GetAccountTransaction(address, seq, include_events)
    }

    pub fn get_account_transactions(
        address: AccountAddress,
        start_seq: u64,
        limit: u64,
        include_events: bool,
    ) -> Self {
        Self::GetAccountTransactions(address, start_seq, limit, include_events)
    }

    pub fn get_events(key: EventKey, start_seq: u64, limit: u64) -> Self {
        Self::GetEvents(key, start_seq, limit)
    }

    pub fn get_currencies() -> Self {
        Self::GetCurrencies([])
    }

    pub fn get_network_status() -> Self {
        Self::GetNetworkStatus([])
    }

    //
    // Experimental APIs
    //

    pub fn get_state_proof(from_version: u64) -> Self {
        Self::GetStateProof((from_version,))
    }

    pub fn get_accumulator_consistency_proof(
        client_known_version: Option<u64>,
        ledger_version: Option<u64>,
    ) -> Self {
        Self::GetAccumulatorConsistencyProof(client_known_version, ledger_version)
    }

    pub fn get_account_state_with_proof(
        address: AccountAddress,
        version: Option<u64>,
        ledger_version: Option<u64>,
    ) -> Self {
        Self::GetAccountStateWithProof(address, version, ledger_version)
    }

    pub fn get_transactions_with_proofs(
        start_version: u64,
        limit: u64,
        include_events: bool,
    ) -> Self {
        Self::GetTransactionsWithProofs(start_version, limit, include_events)
    }

    pub fn get_account_transactions_with_proofs(
        address: AccountAddress,
        start_seq: u64,
        limit: u64,
        include_events: bool,
        ledger_version: Option<u64>,
    ) -> Self {
        Self::GetAccountTransactionsWithProofs(
            address,
            start_seq,
            limit,
            include_events,
            ledger_version,
        )
    }

    pub fn get_events_with_proofs(key: EventKey, start_seq: u64, limit: u64) -> Self {
        Self::GetEventsWithProofs(key, start_seq, limit)
    }

    pub fn get_event_by_version_with_proof(key: EventKey, version: Option<u64>) -> Self {
        Self::GetEventByVersionWithProof(key, version)
    }

    pub fn method(&self) -> Method {
        match self {
            MethodRequest::Submit(_) => Method::Submit,
            MethodRequest::GetMetadata(_) => Method::GetMetadata,
            MethodRequest::GetAccount(_, _) => Method::GetAccount,
            MethodRequest::GetTransactions(_, _, _) => Method::GetTransactions,
            MethodRequest::GetAccountTransaction(_, _, _) => Method::GetAccountTransaction,
            MethodRequest::GetAccountTransactions(_, _, _, _) => Method::GetAccountTransactions,
            MethodRequest::GetEvents(_, _, _) => Method::GetEvents,
            MethodRequest::GetCurrencies(_) => Method::GetCurrencies,
            MethodRequest::GetNetworkStatus(_) => Method::GetNetworkStatus,
            MethodRequest::GetStateProof(_) => Method::GetStateProof,
            MethodRequest::GetAccumulatorConsistencyProof(_, _) => {
                Method::GetAccumulatorConsistencyProof
            }
            MethodRequest::GetAccountStateWithProof(_, _, _) => Method::GetAccountStateWithProof,
            MethodRequest::GetTransactionsWithProofs(_, _, _) => Method::GetTransactionsWithProofs,
            MethodRequest::GetAccountTransactionsWithProofs(_, _, _, _, _) => {
                Method::GetAccountTransactionsWithProofs
            }
            MethodRequest::GetEventsWithProofs(_, _, _) => Method::GetEventsWithProofs,
            MethodRequest::GetEventByVersionWithProof(_, _) => Method::GetEventByVersionWithProof,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    jsonrpc: JsonRpcVersion,
    #[serde(flatten)]
    method_request: MethodRequest,
    id: u64,
}

impl JsonRpcRequest {
    pub fn new(method_request: MethodRequest) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self {
            jsonrpc: JsonRpcVersion::V2,
            method_request,
            id,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn method(&self) -> Method {
        self.method_request.method()
    }
}
