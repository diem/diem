// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::AccountData;
use admission_control_proto::{
    proto::admission_control::SubmitTransactionRequest, proto::AdmissionControlClientBlocking,
    AdmissionControlStatus, SubmitTransactionResponse,
};
use anyhow::{bail, Result};
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::AccountResource,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::{ContractEvent, EventWithProof},
    crypto_proxies::{EpochInfo, LedgerInfoWithSignatures},
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    transaction::{Transaction, Version},
    validator_change::VerifierType,
    vm_error::StatusCode,
    waypoint::Waypoint,
};
use std::convert::TryFrom;

const MAX_GRPC_RETRY_COUNT: u64 = 2;

struct TrustedState {
    version: Version,
    verifier: VerifierType,
    latest_epoch_change_li: Option<LedgerInfoWithSignatures>,
}

/// Struct holding dependencies of client, known_version_and_epoch is updated when learning about
/// new LedgerInfo
pub struct GRPCClient {
    client: AdmissionControlClientBlocking,
    trusted_state: TrustedState,
}

impl GRPCClient {
    /// Construct a new Client instance.
    pub fn new(host: &str, port: u16, waypoint: Option<Waypoint>) -> Result<Self> {
        let client = AdmissionControlClientBlocking::new(host, port);
        // If waypoint is present, use it for initial verification, otherwise the initial
        // verification is essentially empty.
        let (initial_version, initial_verifier) = waypoint.map_or(
            (0, VerifierType::TrustedVerifier(EpochInfo::empty())),
            |w| (w.version(), VerifierType::Waypoint(w)),
        );
        let initial_trusted_state = TrustedState {
            version: initial_version,
            verifier: initial_verifier,
            latest_epoch_change_li: None,
        };
        Ok(GRPCClient {
            client,
            trusted_state: initial_trusted_state,
        })
    }

    /// Submits a transaction and bumps the sequence number for the sender, pass in `None` for
    /// sender_account if sender's address is not managed by the client.
    pub fn submit_transaction(
        &mut self,
        sender_account_opt: Option<&mut AccountData>,
        req: &SubmitTransactionRequest,
    ) -> Result<()> {
        let mut resp = self
            .client
            .submit_transaction(req.clone())
            .map_err(Into::into);

        let mut try_cnt = 0;
        while Self::need_to_retry(try_cnt, &resp) {
            resp = self
                .client
                .submit_transaction(req.clone())
                .map_err(Into::into);
            try_cnt += 1;
        }

        let completed_resp = SubmitTransactionResponse::try_from(resp?)?;

        if let Some(ac_status) = completed_resp.ac_status {
            if ac_status == AdmissionControlStatus::Accepted {
                if let Some(sender_account) = sender_account_opt {
                    // Bump up sequence_number if transaction is accepted.
                    sender_account.sequence_number += 1;
                }
            } else {
                bail!("Transaction failed with AC status: {:?}", ac_status,);
            }
        } else if let Some(vm_error) = completed_resp.vm_error {
            if vm_error.major_status == StatusCode::SEQUENCE_NUMBER_TOO_OLD {
                if let Some(sender_account) = sender_account_opt {
                    sender_account.sequence_number =
                        self.get_sequence_number(sender_account.address)?;
                    bail!(
                        "Transaction failed with vm status: {:?}, please retry your transaction.",
                        vm_error
                    );
                }
            }
            bail!("Transaction failed with vm status: {:?}", vm_error);
        } else if let Some(mempool_error) = completed_resp.mempool_error {
            bail!(
                "Transaction failed with mempool status: {:?}",
                mempool_error,
            );
        } else {
            bail!(
                "Malformed SubmitTransactionResponse which has no status set, {:?}",
                completed_resp,
            );
        }
        Ok(())
    }

    fn get_with_proof(
        &mut self,
        requested_items: Vec<RequestItem>,
    ) -> Result<UpdateToLatestLedgerResponse> {
        let current_trusted_state = &self.trusted_state;
        let req = UpdateToLatestLedgerRequest::new(current_trusted_state.version, requested_items);

        debug!("get_with_proof with request: {:?}", req);
        let proto_req = req.clone().into();
        let resp = self.client.update_to_latest_ledger(proto_req)?;
        let resp = UpdateToLatestLedgerResponse::try_from(resp)?;

        if let Some(new_epoch_info) = resp.verify(&current_trusted_state.verifier, &req)? {
            info!("Trusted epoch change to :{}", new_epoch_info);
            self.trusted_state.verifier = VerifierType::TrustedVerifier(new_epoch_info);
            self.trusted_state.latest_epoch_change_li = resp
                .validator_change_proof
                .ledger_info_with_sigs
                .last()
                .cloned();
        }
        self.trusted_state.version = resp.ledger_info_with_sigs.ledger_info().version();

        Ok(resp)
    }

    fn need_to_retry<T>(try_cnt: u64, ret: &Result<T>) -> bool {
        if try_cnt >= MAX_GRPC_RETRY_COUNT {
            return false;
        }

        if let Err(error) = ret {
            if let Some(grpc_error) = error.downcast_ref::<tonic::Status>() {
                // Only retry when the connection is down to make sure we won't
                // send one txn twice.
                return grpc_error.code() == tonic::Code::Unavailable
                    || grpc_error.code() == tonic::Code::Unknown;
            }
        }

        false
    }

    /// LedgerInfo corresponding to the latest epoch change.
    pub(crate) fn latest_epoch_change_li(&self) -> Option<LedgerInfoWithSignatures> {
        self.trusted_state.latest_epoch_change_li.clone()
    }

    /// Sync version of get_with_proof
    pub(crate) fn get_with_proof_sync(
        &mut self,
        requested_items: Vec<RequestItem>,
    ) -> Result<UpdateToLatestLedgerResponse> {
        let mut resp = self.get_with_proof(requested_items.clone());

        let mut try_cnt = 0;
        while Self::need_to_retry(try_cnt, &resp) {
            resp = self.get_with_proof(requested_items.clone());
            try_cnt += 1;
        }

        resp
    }

    /// Get the latest account sequence number for the account specified.
    pub fn get_sequence_number(&mut self, address: AccountAddress) -> Result<u64> {
        Ok(match self.get_account_blob(address)?.0 {
            Some(blob) => AccountResource::try_from(&blob)?.sequence_number(),
            None => 0,
        })
    }

    /// Get the latest account state blob from validator.
    pub(crate) fn get_account_blob(
        &mut self,
        address: AccountAddress,
    ) -> Result<(Option<AccountStateBlob>, Version)> {
        let req_item = RequestItem::GetAccountState { address };

        let mut response = self.get_with_proof_sync(vec![req_item])?;
        let account_state_with_proof = response
            .response_items
            .remove(0)
            .into_get_account_state_response()?;

        Ok((
            account_state_with_proof.blob,
            response.ledger_info_with_sigs.ledger_info().version(),
        ))
    }

    /// Get transaction from validator by account and sequence number.
    pub fn get_txn_by_acc_seq(
        &mut self,
        account: AccountAddress,
        sequence_number: u64,
        fetch_events: bool,
    ) -> Result<Option<(Transaction, Option<Vec<ContractEvent>>)>> {
        let req_item = RequestItem::GetAccountTransactionBySequenceNumber {
            account,
            sequence_number,
            fetch_events,
        };

        let mut response = self.get_with_proof_sync(vec![req_item])?;
        let (txn_with_proof, _) = response
            .response_items
            .remove(0)
            .into_get_account_txn_by_seq_num_response()?;

        Ok(txn_with_proof.map(|t| (t.transaction, t.events)))
    }

    /// Get transactions in range (start_version..start_version + limit - 1) from validator.
    pub fn get_txn_by_range(
        &mut self,
        start_version: u64,
        limit: u64,
        fetch_events: bool,
    ) -> Result<Vec<(Transaction, Option<Vec<ContractEvent>>)>> {
        // Make the request.
        let req_item = RequestItem::GetTransactions {
            start_version,
            limit,
            fetch_events,
        };
        let mut response = self.get_with_proof_sync(vec![req_item])?;
        let txn_list_with_proof = response
            .response_items
            .remove(0)
            .into_get_transactions_response()?;

        // Transform the response.
        let num_txns = txn_list_with_proof.transactions.len();
        let event_lists = txn_list_with_proof
            .events
            .map(|event_lists| event_lists.into_iter().map(Some).collect())
            .unwrap_or_else(|| vec![None; num_txns]);

        Ok(itertools::zip_eq(txn_list_with_proof.transactions, event_lists).collect())
    }

    /// Get event by access path from validator. AccountStateWithProof will be returned if
    /// 1. No event is available. 2. Ascending and available event number < limit.
    /// 3. Descending and start_seq_num > latest account event sequence number.
    pub fn get_events_by_access_path(
        &mut self,
        access_path: AccessPath,
        start_event_seq_num: u64,
        ascending: bool,
        limit: u64,
    ) -> Result<(Vec<EventWithProof>, AccountStateWithProof)> {
        let req_item = RequestItem::GetEventsByEventAccessPath {
            access_path,
            start_event_seq_num,
            ascending,
            limit,
        };

        let mut response = self.get_with_proof_sync(vec![req_item])?;
        let value_with_proof = response.response_items.remove(0);
        match value_with_proof {
            ResponseItem::GetEventsByEventAccessPath {
                events_with_proof,
                proof_of_latest_event,
            } => Ok((events_with_proof, proof_of_latest_event)),
            _ => bail!(
                "Incorrect type of response returned: {:?}",
                value_with_proof
            ),
        }
    }
}
