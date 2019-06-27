// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::AccountData;
use admission_control_proto::{
    proto::{
        admission_control::{
            SubmitTransactionRequest, SubmitTransactionResponse as ProtoSubmitTransactionResponse,
        },
        admission_control_grpc::AdmissionControlClient,
    },
    AdmissionControlStatus, SubmitTransactionResponse,
};
use failure::prelude::*;
use futures::Future;
use grpcio::{CallOption, ChannelBuilder, EnvBuilder};
use logger::prelude::*;
use proto_conv::{FromProto, IntoProto};
use std::sync::Arc;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::get_account_resource_or_default,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::{ContractEvent, EventWithProof},
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    transaction::{SignedTransaction, Version},
    validator_verifier::ValidatorVerifier,
    vm_error::{VMStatus, VMValidationStatus},
};

const MAX_GRPC_RETRY_COUNT: u64 = 1;

/// Struct holding dependencies of client.
pub struct GRPCClient {
    client: AdmissionControlClient,
    validator_verifier: Arc<ValidatorVerifier>,
}

impl GRPCClient {
    /// Construct a new Client instance.
    pub fn new(host: &str, port: &str, validator_verifier: Arc<ValidatorVerifier>) -> Result<Self> {
        let conn_addr = format!("{}:{}", host, port);

        // Create a GRPC client
        let env = Arc::new(EnvBuilder::new().name_prefix("grpc-client-").build());
        let ch = ChannelBuilder::new(env).connect(&conn_addr);
        let client = AdmissionControlClient::new(ch);

        Ok(GRPCClient {
            client,
            validator_verifier,
        })
    }

    /// Submits a transaction and bumps the sequence number for the sender, pass in `None` for
    /// sender_account if sender's address is not managed by the client.
    pub fn submit_transaction(
        &self,
        sender_account_opt: Option<&mut AccountData>,
        req: &SubmitTransactionRequest,
    ) -> Result<()> {
        let mut resp = self.submit_transaction_opt(req);

        let mut try_cnt = 0_u64;
        while Self::need_to_retry(&mut try_cnt, &resp) {
            resp = self.submit_transaction_opt(&req);
        }

        let completed_resp = SubmitTransactionResponse::from_proto(resp?)?;

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
            if vm_error == VMStatus::Validation(VMValidationStatus::SequenceNumberTooOld) {
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

    /// Async version of submit_transaction
    pub fn submit_transaction_async(
        &self,
        req: &SubmitTransactionRequest,
    ) -> Result<(impl Future<Item = SubmitTransactionResponse, Error = failure::Error>)> {
        let resp = self
            .client
            .submit_transaction_async_opt(&req, Self::get_default_grpc_call_option())?
            .then(|proto_resp| {
                let ret = SubmitTransactionResponse::from_proto(proto_resp?)?;
                Ok(ret)
            });
        Ok(resp)
    }

    fn submit_transaction_opt(
        &self,
        resp: &SubmitTransactionRequest,
    ) -> Result<ProtoSubmitTransactionResponse> {
        Ok(self
            .client
            .submit_transaction_opt(resp, Self::get_default_grpc_call_option())?)
    }

    fn get_with_proof_async(
        &self,
        requested_items: Vec<RequestItem>,
    ) -> Result<impl Future<Item = UpdateToLatestLedgerResponse, Error = failure::Error>> {
        let req = UpdateToLatestLedgerRequest::new(0, requested_items.clone());
        debug!("get_with_proof with request: {:?}", req);
        let proto_req = req.clone().into_proto();
        let arc_validator_verifier: Arc<ValidatorVerifier> = Arc::clone(&self.validator_verifier);
        let ret = self
            .client
            .update_to_latest_ledger_async_opt(&proto_req, Self::get_default_grpc_call_option())?
            .then(move |get_with_proof_resp| {
                // TODO: Cache/persist client_known_version to work with validator set change when
                // the feature is available.

                let resp = UpdateToLatestLedgerResponse::from_proto(get_with_proof_resp?)?;
                resp.verify(arc_validator_verifier, &req)?;
                Ok(resp)
            });
        Ok(ret)
    }

    fn need_to_retry<T>(try_cnt: &mut u64, ret: &Result<T>) -> bool {
        if *try_cnt <= MAX_GRPC_RETRY_COUNT {
            *try_cnt += 1;
            if let Err(error) = ret {
                if let Some(grpc_error) = error.downcast_ref::<grpcio::Error>() {
                    if let grpcio::Error::RpcFailure(grpc_rpc_failure) = grpc_error {
                        // Only retry when the connection is down to make sure we won't
                        // send one txn twice.
                        return grpc_rpc_failure.status == grpcio::RpcStatusCode::Unavailable;
                    }
                }
            }
        }
        false
    }
    /// Sync version of get_with_proof
    pub(crate) fn get_with_proof_sync(
        &self,
        requested_items: Vec<RequestItem>,
    ) -> Result<UpdateToLatestLedgerResponse> {
        let mut resp: Result<UpdateToLatestLedgerResponse> =
            self.get_with_proof_async(requested_items.clone())?.wait();
        let mut try_cnt = 0_u64;

        while Self::need_to_retry(&mut try_cnt, &resp) {
            resp = self.get_with_proof_async(requested_items.clone())?.wait();
        }

        Ok(resp?)
    }

    /// Get the latest account sequence number for the account specified.
    pub fn get_sequence_number(&self, address: AccountAddress) -> Result<u64> {
        Ok(get_account_resource_or_default(&self.get_account_blob(address)?.0)?.sequence_number())
    }

    /// Get the latest account state blob from validator.
    pub(crate) fn get_account_blob(
        &self,
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
        &self,
        account: AccountAddress,
        sequence_number: u64,
        fetch_events: bool,
    ) -> Result<Option<(SignedTransaction, Option<Vec<ContractEvent>>)>> {
        let req_item = RequestItem::GetAccountTransactionBySequenceNumber {
            account,
            sequence_number,
            fetch_events,
        };

        let mut response = self.get_with_proof_sync(vec![req_item])?;
        let (signed_txn_with_proof, _) = response
            .response_items
            .remove(0)
            .into_get_account_txn_by_seq_num_response()?;

        Ok(signed_txn_with_proof.map(|t| (t.signed_transaction, t.events)))
    }

    /// Get transactions in range (start_version..start_version + limit - 1) from validator.
    pub fn get_txn_by_range(
        &self,
        start_version: u64,
        limit: u64,
        fetch_events: bool,
    ) -> Result<Vec<(SignedTransaction, Option<Vec<ContractEvent>>)>> {
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
        let num_txns = txn_list_with_proof.transaction_and_infos.len();
        let event_lists = txn_list_with_proof
            .events
            .map(|event_lists| event_lists.into_iter().map(Some).collect())
            .unwrap_or_else(|| vec![None; num_txns]);

        let res = itertools::zip_eq(txn_list_with_proof.transaction_and_infos, event_lists)
            .map(|((signed_txn, _), events)| (signed_txn, events))
            .collect();
        Ok(res)
    }

    /// Get event by access path from validator. AccountStateWithProof will be returned if
    /// 1. No event is available. 2. Ascending and available event number < limit.
    /// 3. Descending and start_seq_num > latest account event sequence number.
    pub fn get_events_by_access_path(
        &self,
        access_path: AccessPath,
        start_event_seq_num: u64,
        ascending: bool,
        limit: u64,
    ) -> Result<(Vec<EventWithProof>, Option<AccountStateWithProof>)> {
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

    fn get_default_grpc_call_option() -> CallOption {
        CallOption::default()
            .wait_for_ready(true)
            .timeout(std::time::Duration::from_millis(5000))
    }
}
