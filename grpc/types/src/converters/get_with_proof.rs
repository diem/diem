// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{format_err, Error, Result};
use std::convert::{TryFrom, TryInto};

use libra_types::{
    account_address::AccountAddress,
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
};

use crate::proto::types::{
    GetAccountStateRequest, GetAccountStateResponse, GetAccountTransactionBySequenceNumberRequest,
    GetAccountTransactionBySequenceNumberResponse, GetEventsByEventAccessPathRequest,
    GetEventsByEventAccessPathResponse, GetTransactionsRequest, GetTransactionsResponse,
};

impl TryFrom<crate::proto::types::UpdateToLatestLedgerRequest> for UpdateToLatestLedgerRequest {
    type Error = Error;

    fn try_from(proto: crate::proto::types::UpdateToLatestLedgerRequest) -> Result<Self> {
        Ok(Self {
            client_known_version: proto.client_known_version,
            requested_items: proto
                .requested_items
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl From<UpdateToLatestLedgerRequest> for crate::proto::types::UpdateToLatestLedgerRequest {
    fn from(request: UpdateToLatestLedgerRequest) -> Self {
        Self {
            client_known_version: request.client_known_version,
            requested_items: request
                .requested_items
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<crate::proto::types::UpdateToLatestLedgerResponse> for UpdateToLatestLedgerResponse {
    type Error = Error;

    fn try_from(proto: crate::proto::types::UpdateToLatestLedgerResponse) -> Result<Self> {
        let response_items = proto
            .response_items
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;
        let ledger_info_with_sigs = proto
            .ledger_info_with_sigs
            .unwrap_or_else(Default::default)
            .try_into()?;
        let epoch_change_proof = proto
            .epoch_change_proof
            .unwrap_or_else(Default::default)
            .try_into()?;
        let ledger_consistency_proof = proto
            .ledger_consistency_proof
            .unwrap_or_else(Default::default)
            .try_into()?;

        Ok(Self {
            response_items,
            ledger_info_with_sigs,
            epoch_change_proof,
            ledger_consistency_proof,
        })
    }
}

impl From<UpdateToLatestLedgerResponse> for crate::proto::types::UpdateToLatestLedgerResponse {
    fn from(response: UpdateToLatestLedgerResponse) -> Self {
        let response_items = response
            .response_items
            .into_iter()
            .map(Into::into)
            .collect();
        let ledger_info_with_sigs = Some(response.ledger_info_with_sigs.into());
        let epoch_change_proof = Some(response.epoch_change_proof.into());
        let ledger_consistency_proof = Some(response.ledger_consistency_proof.into());

        Self {
            response_items,
            ledger_info_with_sigs,
            epoch_change_proof,
            ledger_consistency_proof,
        }
    }
}

impl TryFrom<crate::proto::types::RequestItem> for RequestItem {
    type Error = Error;

    fn try_from(proto: crate::proto::types::RequestItem) -> Result<Self> {
        use crate::proto::types::request_item::RequestedItems::*;

        let item = proto
            .requested_items
            .ok_or_else(|| format_err!("Missing requested_items"))?;

        let request = match item {
            GetAccountStateRequest(request) => {
                let address = AccountAddress::try_from(request.address)?;
                RequestItem::GetAccountState { address }
            }
            GetAccountTransactionBySequenceNumberRequest(request) => {
                let account = AccountAddress::try_from(request.account)?;
                let sequence_number = request.sequence_number;
                let fetch_events = request.fetch_events;

                RequestItem::GetAccountTransactionBySequenceNumber {
                    account,
                    sequence_number,
                    fetch_events,
                }
            }
            GetEventsByEventAccessPathRequest(request) => {
                let access_path = request
                    .access_path
                    .ok_or_else(|| format_err!("Missing access_path"))?
                    .try_into()?;
                let start_event_seq_num = request.start_event_seq_num;
                let ascending = request.ascending;
                let limit = request.limit;

                RequestItem::GetEventsByEventAccessPath {
                    access_path,
                    start_event_seq_num,
                    ascending,
                    limit,
                }
            }
            GetTransactionsRequest(request) => {
                let start_version = request.start_version;
                let limit = request.limit;
                let fetch_events = request.fetch_events;

                RequestItem::GetTransactions {
                    start_version,
                    limit,
                    fetch_events,
                }
            }
        };

        Ok(request)
    }
}

impl From<RequestItem> for crate::proto::types::RequestItem {
    fn from(request: RequestItem) -> Self {
        use crate::proto::types::request_item::RequestedItems;

        let req = match request {
            RequestItem::GetAccountState { address } => {
                RequestedItems::GetAccountStateRequest(GetAccountStateRequest {
                    address: address.into(),
                })
            }
            RequestItem::GetAccountTransactionBySequenceNumber {
                account,
                sequence_number,
                fetch_events,
            } => RequestedItems::GetAccountTransactionBySequenceNumberRequest(
                GetAccountTransactionBySequenceNumberRequest {
                    account: account.into(),
                    sequence_number,
                    fetch_events,
                },
            ),
            RequestItem::GetEventsByEventAccessPath {
                access_path,
                start_event_seq_num,
                ascending,
                limit,
            } => RequestedItems::GetEventsByEventAccessPathRequest(
                GetEventsByEventAccessPathRequest {
                    access_path: Some(access_path.into()),
                    start_event_seq_num,
                    ascending,
                    limit,
                },
            ),
            RequestItem::GetTransactions {
                start_version,
                limit,
                fetch_events,
            } => RequestedItems::GetTransactionsRequest(GetTransactionsRequest {
                start_version,
                limit,
                fetch_events,
            }),
        };

        Self {
            requested_items: Some(req),
        }
    }
}

impl TryFrom<crate::proto::types::ResponseItem> for ResponseItem {
    type Error = Error;

    fn try_from(proto: crate::proto::types::ResponseItem) -> Result<Self> {
        use crate::proto::types::response_item::ResponseItems::*;

        let item = proto
            .response_items
            .ok_or_else(|| format_err!("Missing response_items"))?;

        let response = match item {
            GetAccountStateResponse(response) => {
                let account_state_with_proof = response
                    .account_state_with_proof
                    .ok_or_else(|| format_err!("Missing account_state_with_proof"))?
                    .try_into()?;
                ResponseItem::GetAccountState {
                    account_state_with_proof,
                }
            }
            GetAccountTransactionBySequenceNumberResponse(response) => {
                let transaction_with_proof = response
                    .transaction_with_proof
                    .map(TryInto::try_into)
                    .transpose()?;
                let proof_of_current_sequence_number = response
                    .proof_of_current_sequence_number
                    .map(TryInto::try_into)
                    .transpose()?;

                ResponseItem::GetAccountTransactionBySequenceNumber {
                    transaction_with_proof,
                    proof_of_current_sequence_number,
                }
            }
            GetEventsByEventAccessPathResponse(response) => {
                let events_with_proof = response
                    .events_with_proof
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<Vec<_>>>()?;
                let proof_of_latest_event = response
                    .proof_of_latest_event
                    .ok_or_else(|| format_err!("Missing proof_of_latest_event"))?
                    .try_into()?;

                ResponseItem::GetEventsByEventAccessPath {
                    events_with_proof,
                    proof_of_latest_event,
                }
            }
            GetTransactionsResponse(response) => {
                let txn_list_with_proof = response
                    .txn_list_with_proof
                    .ok_or_else(|| format_err!("Missing txn_list_with_proof"))?
                    .try_into()?;

                ResponseItem::GetTransactions {
                    txn_list_with_proof,
                }
            }
        };

        Ok(response)
    }
}

impl From<ResponseItem> for crate::proto::types::ResponseItem {
    fn from(response: ResponseItem) -> Self {
        use crate::proto::types::response_item::ResponseItems;

        let res = match response {
            ResponseItem::GetAccountState {
                account_state_with_proof,
            } => ResponseItems::GetAccountStateResponse(GetAccountStateResponse {
                account_state_with_proof: Some(account_state_with_proof.into()),
            }),
            ResponseItem::GetAccountTransactionBySequenceNumber {
                transaction_with_proof,
                proof_of_current_sequence_number,
            } => ResponseItems::GetAccountTransactionBySequenceNumberResponse(
                GetAccountTransactionBySequenceNumberResponse {
                    transaction_with_proof: transaction_with_proof.map(Into::into),
                    proof_of_current_sequence_number: proof_of_current_sequence_number
                        .map(Into::into),
                },
            ),
            ResponseItem::GetEventsByEventAccessPath {
                events_with_proof,
                proof_of_latest_event,
            } => ResponseItems::GetEventsByEventAccessPathResponse(
                GetEventsByEventAccessPathResponse {
                    events_with_proof: events_with_proof.into_iter().map(Into::into).collect(),
                    proof_of_latest_event: Some(proof_of_latest_event.into()),
                },
            ),
            ResponseItem::GetTransactions {
                txn_list_with_proof,
            } => ResponseItems::GetTransactionsResponse(GetTransactionsResponse {
                txn_list_with_proof: Some(txn_list_with_proof.into()),
            }),
        };

        Self {
            response_items: Some(res),
        }
    }
}
