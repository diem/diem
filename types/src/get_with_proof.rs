// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::get_account_resource_or_default,
    account_state_blob::AccountStateWithProof,
    contract_event::EventWithProof,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::AccumulatorConsistencyProof,
    proto::get_with_proof::{
        GetAccountStateRequest, GetAccountStateResponse,
        GetAccountTransactionBySequenceNumberRequest,
        GetAccountTransactionBySequenceNumberResponse, GetEventsByEventAccessPathRequest,
        GetEventsByEventAccessPathResponse, GetTransactionsRequest, GetTransactionsResponse,
    },
    transaction::{SignedTransactionWithProof, TransactionListWithProof, Version},
    validator_change::ValidatorChangeEventWithProof,
    validator_verifier::ValidatorVerifier,
};
use crypto::{hash::CryptoHash, *};
use failure::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use std::{
    cmp,
    convert::{TryFrom, TryInto},
    mem,
    sync::Arc,
};

#[derive(Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[ProtoType(crate::proto::get_with_proof::UpdateToLatestLedgerRequest)]
pub struct UpdateToLatestLedgerRequest {
    pub client_known_version: u64,
    pub requested_items: Vec<RequestItem>,
}

impl UpdateToLatestLedgerRequest {
    pub fn new(client_known_version: u64, requested_items: Vec<RequestItem>) -> Self {
        UpdateToLatestLedgerRequest {
            client_known_version,
            requested_items,
        }
    }
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateToLatestLedgerResponse<Sig> {
    pub response_items: Vec<ResponseItem>,
    pub ledger_info_with_sigs: LedgerInfoWithSignatures<Sig>,
    pub validator_change_events: Vec<ValidatorChangeEventWithProof<Sig>>,
    pub ledger_consistency_proof: AccumulatorConsistencyProof,
}

impl<Sig: Signature> IntoProto for UpdateToLatestLedgerResponse<Sig> {
    type ProtoType = crate::proto::get_with_proof::UpdateToLatestLedgerResponse;

    fn into_proto(self) -> Self::ProtoType {
        let mut out = crate::proto::get_with_proof::UpdateToLatestLedgerResponse::new();
        out.set_response_items(self.response_items.into_proto());
        out.set_ledger_info_with_sigs(self.ledger_info_with_sigs.into_proto());
        out.set_validator_change_events(self.validator_change_events.into_proto());
        out.set_ledger_consistency_proof(self.ledger_consistency_proof.into_proto());
        out
    }
}

impl<Sig: Signature> FromProto for UpdateToLatestLedgerResponse<Sig> {
    type ProtoType = crate::proto::get_with_proof::UpdateToLatestLedgerResponse;

    fn from_proto(mut object: Self::ProtoType) -> failure::Result<Self> {
        Ok(UpdateToLatestLedgerResponse {
            response_items: <Vec<ResponseItem> as FromProto>::from_proto(
                object.take_response_items(),
            )?,
            ledger_info_with_sigs: LedgerInfoWithSignatures::from_proto(
                object.take_ledger_info_with_sigs(),
            )?,
            validator_change_events:
                <Vec<ValidatorChangeEventWithProof<Sig>> as FromProto>::from_proto(
                    object.take_validator_change_events(),
                )?,
            ledger_consistency_proof: AccumulatorConsistencyProof::from_proto(
                object.take_ledger_consistency_proof(),
            )?,
        })
    }
}

impl<Sig: Signature> TryFrom<crate::proto::types::UpdateToLatestLedgerResponse>
    for UpdateToLatestLedgerResponse<Sig>
{
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
        let validator_change_events = proto
            .validator_change_events
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;
        let ledger_consistency_proof = proto
            .ledger_consistency_proof
            .unwrap_or_else(Default::default)
            .try_into()?;

        Ok(Self {
            response_items,
            ledger_info_with_sigs,
            validator_change_events,
            ledger_consistency_proof,
        })
    }
}

impl<Sig: Signature> From<UpdateToLatestLedgerResponse<Sig>>
    for crate::proto::types::UpdateToLatestLedgerResponse
{
    fn from(response: UpdateToLatestLedgerResponse<Sig>) -> Self {
        let response_items = response
            .response_items
            .into_iter()
            .map(Into::into)
            .collect();
        let ledger_info_with_sigs = Some(response.ledger_info_with_sigs.into());
        let validator_change_events = response
            .validator_change_events
            .into_iter()
            .map(Into::into)
            .collect();
        let ledger_consistency_proof = Some(response.ledger_consistency_proof.into());

        Self {
            response_items,
            ledger_info_with_sigs,
            validator_change_events,
            ledger_consistency_proof,
        }
    }
}

impl<Sig: Signature> UpdateToLatestLedgerResponse<Sig> {
    /// Constructor.
    pub fn new(
        response_items: Vec<ResponseItem>,
        ledger_info_with_sigs: LedgerInfoWithSignatures<Sig>,
        validator_change_events: Vec<ValidatorChangeEventWithProof<Sig>>,
        ledger_consistency_proof: AccumulatorConsistencyProof,
    ) -> Self {
        UpdateToLatestLedgerResponse {
            response_items,
            ledger_info_with_sigs,
            validator_change_events,
            ledger_consistency_proof,
        }
    }

    /// Verifies that the response has items corresponding to request items and each of them are
    /// supported by proof it carries and is what the request item asks for.
    ///
    /// After calling this one can trust the info in the response items without further
    /// verification.
    pub fn verify(
        &self,
        validator_verifier: Arc<ValidatorVerifier<Sig::VerifyingKeyMaterial>>,
        request: &UpdateToLatestLedgerRequest,
    ) -> Result<()> {
        verify_update_to_latest_ledger_response(
            validator_verifier,
            request.client_known_version,
            &request.requested_items,
            &self.response_items,
            &self.ledger_info_with_sigs,
        )
    }
}

/// Verifies content of an [`UpdateToLatestLedgerResponse`] against the proofs it
/// carries and the content of the corresponding [`UpdateToLatestLedgerRequest`]
pub fn verify_update_to_latest_ledger_response<Sig: Signature>(
    validator_verifier: Arc<ValidatorVerifier<Sig::VerifyingKeyMaterial>>,
    req_client_known_version: u64,
    req_request_items: &[RequestItem],
    response_items: &[ResponseItem],
    ledger_info_with_sigs: &LedgerInfoWithSignatures<Sig>,
) -> Result<()> {
    let (ledger_info, signatures) = (
        ledger_info_with_sigs.ledger_info(),
        ledger_info_with_sigs.signatures(),
    );

    // Verify that the same or a newer ledger info is returned.
    ensure!(
        ledger_info.version() >= req_client_known_version,
        "Got stale ledger_info with version {}, known version: {}.",
        ledger_info.version(),
        req_client_known_version,
    );

    // Verify ledger info signatures.
    if !(ledger_info.version() == 0 && signatures.is_empty()) {
        validator_verifier.batch_verify_aggregated_signature(ledger_info.hash(), signatures)?;
    }

    // Verify each sub response.
    ensure!(
        req_request_items.len() == response_items.len(),
        "Number of request items ({}) does not match that of response items ({}).",
        req_request_items.len(),
        response_items.len(),
    );
    itertools::zip_eq(req_request_items, response_items)
        .map(|(req, res)| verify_response_item(ledger_info, req, res))
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

fn verify_response_item(
    ledger_info: &LedgerInfo,
    req: &RequestItem,
    res: &ResponseItem,
) -> Result<()> {
    match (req, res) {
        // GetAccountState
        (
            RequestItem::GetAccountState { address },
            ResponseItem::GetAccountState {
                account_state_with_proof,
            },
        ) => account_state_with_proof.verify(ledger_info, ledger_info.version(), *address),
        // GetAccountTransactionBySequenceNumber
        (
            RequestItem::GetAccountTransactionBySequenceNumber {
                account,
                sequence_number,
                fetch_events,
            },
            ResponseItem::GetAccountTransactionBySequenceNumber {
                signed_transaction_with_proof,
                proof_of_current_sequence_number,
            },
        ) => verify_get_txn_by_seq_num_resp(
            ledger_info,
            *account,
            *sequence_number,
            *fetch_events,
            signed_transaction_with_proof.as_ref(),
            proof_of_current_sequence_number.as_ref(),
        ),
        // GetEventsByEventAccessPath
        (
            RequestItem::GetEventsByEventAccessPath {
                access_path,
                start_event_seq_num,
                ascending,
                limit,
            },
            ResponseItem::GetEventsByEventAccessPath {
                events_with_proof,
                proof_of_latest_event,
            },
        ) => verify_get_events_by_access_path_resp(
            ledger_info,
            access_path,
            *start_event_seq_num,
            *ascending,
            *limit,
            events_with_proof,
            proof_of_latest_event,
        ),
        // GetTransactions
        (
            RequestItem::GetTransactions {
                start_version,
                limit,
                fetch_events,
            },
            ResponseItem::GetTransactions {
                txn_list_with_proof,
            },
        ) => verify_get_txns_resp(
            ledger_info,
            *start_version,
            *limit,
            *fetch_events,
            txn_list_with_proof,
        ),
        // Request-response item types mismatch.
        _ => bail!(
            "RequestItem/ResponseItem types mismatch. request: {:?}, response: {:?}",
            mem::discriminant(req),
            mem::discriminant(res),
        ),
    }
}

fn verify_get_txn_by_seq_num_resp(
    ledger_info: &LedgerInfo,
    req_account: AccountAddress,
    req_sequence_number: u64,
    req_fetch_events: bool,
    signed_transaction_with_proof: Option<&SignedTransactionWithProof>,
    proof_of_current_sequence_number: Option<&AccountStateWithProof>,
) -> Result<()> {
    match (signed_transaction_with_proof, proof_of_current_sequence_number) {
        (Some(signed_transaction_with_proof), None) => {
            ensure!(
                req_fetch_events == signed_transaction_with_proof.events.is_some(),
                "Bad GetAccountTxnBySeqNum response. Events requested: {}, events returned: {}.",
                req_fetch_events,
                signed_transaction_with_proof.events.is_some(),
            );
            signed_transaction_with_proof.verify(
                ledger_info,
                signed_transaction_with_proof.version,
                req_account,
                req_sequence_number,
            )
        },
        (None, Some(proof_of_current_sequence_number)) => {
            let sequence_number_in_ledger =
                get_account_resource_or_default(&proof_of_current_sequence_number.blob)?
                    .sequence_number();
            ensure!(
                sequence_number_in_ledger <= req_sequence_number,
                "Server returned no transactions while it should. Seq num requested: {}, latest seq num in ledger: {}.",
                req_sequence_number,
                sequence_number_in_ledger
            );
            proof_of_current_sequence_number.verify(ledger_info, ledger_info.version(), req_account)
        },
        _ => bail!(
            "Bad GetAccountTxnBySeqNum response. txn_proof.is_none():{}, cur_seq_num_proof.is_none():{}",
            signed_transaction_with_proof.is_none(),
            proof_of_current_sequence_number.is_none(),
        )
    }
}

fn verify_get_events_by_access_path_resp(
    ledger_info: &LedgerInfo,
    req_access_path: &AccessPath,
    req_start_seq_num: u64,
    req_ascending: bool,
    req_limit: u64,
    events_with_proof: &[EventWithProof],
    proof_of_latest_event: &AccountStateWithProof,
) -> Result<()> {
    let account_resource = get_account_resource_or_default(&proof_of_latest_event.blob)?;
    let (seq_num_upper_bound, expected_event_key) = {
        proof_of_latest_event.verify(
            ledger_info,
            ledger_info.version(),
            req_access_path.address,
        )?;
        let event_handle =
            account_resource.get_event_handle_by_query_path(&req_access_path.path)?;
        (event_handle.count(), event_handle.key())
    };

    let cursor =
        if !req_ascending && req_start_seq_num == u64::max_value() && seq_num_upper_bound > 0 {
            seq_num_upper_bound - 1
        } else {
            req_start_seq_num
        };

    let expected_seq_nums = if cursor >= seq_num_upper_bound {
        // Unreachable, so empty.
        Vec::new()
    } else if req_ascending {
        // Ascending, from start to upper bound or limit.
        (cursor..cmp::min(cursor + req_limit, seq_num_upper_bound)).collect()
    } else if cursor + 1 < req_limit {
        // Descending and hitting 0.
        (0..=cursor).rev().collect()
    } else {
        // Descending and hitting limit.
        (cursor + 1 - req_limit..=cursor).rev().collect()
    };

    ensure!(
        expected_seq_nums.len() == events_with_proof.len(),
        "Expecting {} events, got {}.",
        expected_seq_nums.len(),
        events_with_proof.len(),
    );
    itertools::zip_eq(events_with_proof, expected_seq_nums)
        .map(|(e, seq_num)| {
            e.verify(
                ledger_info,
                expected_event_key,
                seq_num,
                e.transaction_version,
                e.event_index,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

fn verify_get_txns_resp(
    ledger_info: &LedgerInfo,
    req_start_version: Version,
    req_limit: u64,
    req_fetch_events: bool,
    txn_list_with_proof: &TransactionListWithProof,
) -> Result<()> {
    ensure!(
        req_fetch_events == txn_list_with_proof.events.is_some(),
        "Bad GetTransactions response. Events requested: {}, events returned: {}.",
        req_fetch_events,
        txn_list_with_proof.events.is_some(),
    );

    if req_limit == 0 || req_start_version > ledger_info.version() {
        txn_list_with_proof.verify(ledger_info, None)
    } else {
        let num_txns = txn_list_with_proof.transaction_and_infos.len();
        ensure!(
            cmp::min(req_limit, ledger_info.version() - req_start_version + 1)
                == txn_list_with_proof.transaction_and_infos.len() as u64,
            "Number of transactions returned not expected. num_txns: {}, start version: {}, latest version: {}",
            num_txns,
            req_start_version,
            ledger_info.version(),
        );
        txn_list_with_proof.verify(ledger_info, Some(req_start_version))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub enum RequestItem {
    GetAccountTransactionBySequenceNumber {
        account: AccountAddress,
        sequence_number: u64,
        fetch_events: bool,
    },
    // this can't be the first variant, tracked here https://github.com/AltSysrq/proptest/issues/141
    GetAccountState {
        address: AccountAddress,
    },
    GetEventsByEventAccessPath {
        access_path: AccessPath,
        start_event_seq_num: u64,
        ascending: bool,
        limit: u64,
    },
    GetTransactions {
        start_version: Version,
        limit: u64,
        fetch_events: bool,
    },
}

impl FromProto for RequestItem {
    type ProtoType = crate::proto::get_with_proof::RequestItem;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        Ok(if object.has_get_account_state_request() {
            let address =
                AccountAddress::from_proto(object.take_get_account_state_request().take_address())?;
            RequestItem::GetAccountState { address }
        } else if object.has_get_account_transaction_by_sequence_number_request() {
            let mut req = object.take_get_account_transaction_by_sequence_number_request();
            let account = AccountAddress::from_proto(req.take_account())?;
            let sequence_number = req.get_sequence_number();
            let fetch_events = req.get_fetch_events();

            RequestItem::GetAccountTransactionBySequenceNumber {
                account,
                sequence_number,
                fetch_events,
            }
        } else if object.has_get_events_by_event_access_path_request() {
            let mut req = object.take_get_events_by_event_access_path_request();

            let access_path = AccessPath::from_proto(req.take_access_path())?;
            let start_event_seq_num = req.get_start_event_seq_num();
            let ascending = req.get_ascending();
            let limit = req.get_limit();

            RequestItem::GetEventsByEventAccessPath {
                access_path,
                start_event_seq_num,
                ascending,
                limit,
            }
        } else if object.has_get_transactions_request() {
            let req = object.get_get_transactions_request();
            let start_version = req.get_start_version();
            let limit = req.get_limit();
            let fetch_events = req.get_fetch_events();

            RequestItem::GetTransactions {
                start_version,
                limit,
                fetch_events,
            }
        } else {
            bail!("Unknown RequestItem type.")
        })
    }
}

impl IntoProto for RequestItem {
    type ProtoType = crate::proto::get_with_proof::RequestItem;

    fn into_proto(self) -> Self::ProtoType {
        let mut out = Self::ProtoType::new();
        match self {
            RequestItem::GetAccountState { address } => {
                let mut req = GetAccountStateRequest::new();
                req.set_address(address.into_proto());
                out.set_get_account_state_request(req);
            }
            RequestItem::GetAccountTransactionBySequenceNumber {
                account,
                sequence_number,
                fetch_events,
            } => {
                let mut req = GetAccountTransactionBySequenceNumberRequest::new();
                req.set_account(account.into_proto());
                req.set_sequence_number(sequence_number);
                req.set_fetch_events(fetch_events);

                out.set_get_account_transaction_by_sequence_number_request(req);
            }
            RequestItem::GetEventsByEventAccessPath {
                access_path,
                start_event_seq_num,
                ascending,
                limit,
            } => {
                let mut req = GetEventsByEventAccessPathRequest::new();
                req.set_access_path(access_path.into_proto());
                req.set_start_event_seq_num(start_event_seq_num);
                req.set_ascending(ascending);
                req.set_limit(limit);

                out.set_get_events_by_event_access_path_request(req);
            }
            RequestItem::GetTransactions {
                start_version,
                limit,
                fetch_events,
            } => {
                let mut req = GetTransactionsRequest::new();
                req.set_start_version(start_version);
                req.set_limit(limit);
                req.set_fetch_events(fetch_events);

                out.set_get_transactions_request(req);
            }
        }
        out
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
        use crate::proto::types::{
            GetAccountStateRequest, GetAccountTransactionBySequenceNumberRequest,
            GetEventsByEventAccessPathRequest, GetTransactionsRequest,
        };

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

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub enum ResponseItem {
    GetAccountTransactionBySequenceNumber {
        signed_transaction_with_proof: Option<SignedTransactionWithProof>,
        proof_of_current_sequence_number: Option<AccountStateWithProof>,
    },
    // this can't be the first variant, tracked here https://github.com/AltSysrq/proptest/issues/141
    GetAccountState {
        account_state_with_proof: AccountStateWithProof,
    },
    GetEventsByEventAccessPath {
        events_with_proof: Vec<EventWithProof>,
        // TODO: Rename this field to proof_of_event_handle.
        proof_of_latest_event: AccountStateWithProof,
    },
    GetTransactions {
        txn_list_with_proof: TransactionListWithProof,
    },
}

impl ResponseItem {
    pub fn into_get_account_state_response(self) -> Result<AccountStateWithProof> {
        match self {
            ResponseItem::GetAccountState {
                account_state_with_proof,
            } => Ok(account_state_with_proof),
            _ => bail!("Not ResponseItem::GetAccountState."),
        }
    }

    pub fn into_get_account_txn_by_seq_num_response(
        self,
    ) -> Result<(
        Option<SignedTransactionWithProof>,
        Option<AccountStateWithProof>,
    )> {
        match self {
            ResponseItem::GetAccountTransactionBySequenceNumber {
                signed_transaction_with_proof,
                proof_of_current_sequence_number,
            } => Ok((
                signed_transaction_with_proof,
                proof_of_current_sequence_number,
            )),
            _ => bail!("Not ResponseItem::GetAccountTransactionBySequenceNumber."),
        }
    }

    pub fn into_get_events_by_access_path_response(
        self,
    ) -> Result<(Vec<EventWithProof>, AccountStateWithProof)> {
        match self {
            ResponseItem::GetEventsByEventAccessPath {
                events_with_proof,
                proof_of_latest_event,
            } => Ok((events_with_proof, proof_of_latest_event)),
            _ => bail!("Not ResponseItem::GetEventsByEventAccessPath."),
        }
    }

    pub fn into_get_transactions_response(self) -> Result<TransactionListWithProof> {
        match self {
            ResponseItem::GetTransactions {
                txn_list_with_proof,
            } => Ok(txn_list_with_proof),
            _ => bail!("Not ResponseItem::GetTransactions."),
        }
    }
}

impl FromProto for ResponseItem {
    type ProtoType = crate::proto::get_with_proof::ResponseItem;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        Ok(if object.has_get_account_state_response() {
            let account_state_with_proof = AccountStateWithProof::from_proto(
                object
                    .take_get_account_state_response()
                    .take_account_state_with_proof(),
            )?;

            ResponseItem::GetAccountState {
                account_state_with_proof,
            }
        } else if object.has_get_account_transaction_by_sequence_number_response() {
            let mut res = object.take_get_account_transaction_by_sequence_number_response();
            let signed_transaction_with_proof = res
                .signed_transaction_with_proof
                .take()
                .map(SignedTransactionWithProof::from_proto)
                .transpose()?;
            let proof_of_current_sequence_number = res
                .proof_of_current_sequence_number
                .take()
                .map(AccountStateWithProof::from_proto)
                .transpose()?;

            ResponseItem::GetAccountTransactionBySequenceNumber {
                signed_transaction_with_proof,
                proof_of_current_sequence_number,
            }
        } else if object.has_get_events_by_event_access_path_response() {
            let mut res = object.take_get_events_by_event_access_path_response();

            let events_with_proof = res
                .take_events_with_proof()
                .into_iter()
                .map(EventWithProof::from_proto)
                .collect::<Result<Vec<_>>>()?;

            let proof_of_latest_event =
                AccountStateWithProof::from_proto(res.take_proof_of_latest_event())?;

            ResponseItem::GetEventsByEventAccessPath {
                events_with_proof,
                proof_of_latest_event,
            }
        } else if object.has_get_transactions_response() {
            let mut res = object.take_get_transactions_response();
            let txn_list_with_proof =
                TransactionListWithProof::from_proto(res.take_txn_list_with_proof())?;

            ResponseItem::GetTransactions {
                txn_list_with_proof,
            }
        } else {
            bail!("Unknown ResponseItem type.")
        })
    }
}

impl IntoProto for ResponseItem {
    type ProtoType = crate::proto::get_with_proof::ResponseItem;

    fn into_proto(self) -> Self::ProtoType {
        let mut out = Self::ProtoType::new();
        match self {
            ResponseItem::GetAccountState {
                account_state_with_proof,
            } => {
                let mut res = GetAccountStateResponse::new();
                res.set_account_state_with_proof(account_state_with_proof.into_proto());

                out.set_get_account_state_response(res);
            }
            ResponseItem::GetAccountTransactionBySequenceNumber {
                signed_transaction_with_proof,
                proof_of_current_sequence_number,
            } => {
                let mut res = GetAccountTransactionBySequenceNumberResponse::new();

                if let Some(t) = signed_transaction_with_proof {
                    res.set_signed_transaction_with_proof(t.into_proto())
                }
                if let Some(p) = proof_of_current_sequence_number {
                    res.set_proof_of_current_sequence_number(p.into_proto())
                }

                out.set_get_account_transaction_by_sequence_number_response(res);
            }
            ResponseItem::GetEventsByEventAccessPath {
                events_with_proof,
                proof_of_latest_event,
            } => {
                let mut res = GetEventsByEventAccessPathResponse::new();
                res.set_events_with_proof(::protobuf::RepeatedField::from_vec(
                    events_with_proof
                        .into_iter()
                        .map(EventWithProof::into_proto)
                        .collect(),
                ));
                res.set_proof_of_latest_event(proof_of_latest_event.into_proto());

                out.set_get_events_by_event_access_path_response(res);
            }
            ResponseItem::GetTransactions {
                txn_list_with_proof,
            } => {
                let mut res = GetTransactionsResponse::new();
                res.set_txn_list_with_proof(txn_list_with_proof.into_proto());

                out.set_get_transactions_response(res)
            }
        }
        out
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
                let signed_transaction_with_proof = response
                    .signed_transaction_with_proof
                    .map(TryInto::try_into)
                    .transpose()?;
                let proof_of_current_sequence_number = response
                    .proof_of_current_sequence_number
                    .map(TryInto::try_into)
                    .transpose()?;

                ResponseItem::GetAccountTransactionBySequenceNumber {
                    signed_transaction_with_proof,
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
        use crate::proto::types::{
            GetAccountStateResponse, GetAccountTransactionBySequenceNumberResponse,
            GetEventsByEventAccessPathResponse, GetTransactionsResponse,
        };

        let res = match response {
            ResponseItem::GetAccountState {
                account_state_with_proof,
            } => ResponseItems::GetAccountStateResponse(GetAccountStateResponse {
                account_state_with_proof: Some(account_state_with_proof.into()),
            }),
            ResponseItem::GetAccountTransactionBySequenceNumber {
                signed_transaction_with_proof,
                proof_of_current_sequence_number,
            } => ResponseItems::GetAccountTransactionBySequenceNumberResponse(
                GetAccountTransactionBySequenceNumberResponse {
                    signed_transaction_with_proof: signed_transaction_with_proof.map(Into::into),
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
