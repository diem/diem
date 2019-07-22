// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use admission_control_proto::proto::{
    admission_control::{
        AdmissionControlStatusCode, SubmitTransactionRequest,
        SubmitTransactionResponse as ProtoSubmitTransactionResponse,
    },
    admission_control_grpc::AdmissionControlClient,
};
use client::AccountStatus;
use failure::prelude::*;
use futures::{
    stream::{self, Stream},
    Future,
};
use grpcio::{self, CallOption};
use logger::prelude::*;
use proto_conv::{FromProto, IntoProto};
use std::slice::Chunks;
use types::{
    account_address::AccountAddress,
    account_config::get_account_resource_or_default,
    get_with_proof::{RequestItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse},
};

use crate::OP_COUNTER;

/// Timeout duration for grpc call option.
const GRPC_TIMEOUT_MS: u64 = 8_000;

/// Return a parameter that controls how "patient" AC clients are,
/// who are waiting the response from AC for this amount of time.
fn get_default_grpc_call_option() -> CallOption {
    CallOption::default()
        .wait_for_ready(true)
        .timeout(std::time::Duration::from_millis(GRPC_TIMEOUT_MS))
}

/// Divide generic items into a vector of chunks of nearly equal size.
pub fn divide_items<T>(items: &[T], num_chunks: usize) -> Chunks<T> {
    let chunk_size = if (num_chunks == 0) || (items.len() / num_chunks == 0) {
        std::cmp::max(1, items.len())
    } else {
        items.len() / num_chunks
    };
    items.chunks(chunk_size)
}

/// ---------------------------------------------------------- ///
///  Transaction async request and response handling helpers.  ///
/// ---------------------------------------------------------- ///

/// By checking 1) ac status, 2) vm status, and 3) mempool status, decide whether the reponse
/// from AC is accepted. If not, classify what the error type is.
fn check_ac_response(resp: &ProtoSubmitTransactionResponse) -> bool {
    if resp.has_ac_status() {
        let status = resp.get_ac_status().get_code();
        if status == AdmissionControlStatusCode::Accepted {
            OP_COUNTER.inc(&format!("submit_txns.{:?}", status));
            true
        } else {
            OP_COUNTER.inc(&format!("submit_txns.{:?}", status));
            error!("Request rejected by AC: {:?}", resp);
            false
        }
    } else if resp.has_vm_status() {
        OP_COUNTER.inc(&format!("submit_txns.{:?}", resp.get_vm_status()));
        error!("Request causes error on VM: {:?}", resp);
        false
    } else if resp.has_mempool_status() {
        OP_COUNTER.inc(&format!(
            "submit_txns.{:?}",
            resp.get_mempool_status().get_code()
        ));
        error!("Request causes error on mempool: {:?}", resp);
        false
    } else {
        OP_COUNTER.inc("submit_txns.Unknown");
        error!("Request rejected by AC for unknown error: {:?}", resp);
        false
    }
}

/// Send TXN requests to AC async, wait for and check the responses from AC.
/// Return only the responses of accepted TXN requests.
/// Ignore but count both gRPC-failed submissions and AC-rejected TXNs.
pub fn submit_and_wait_txn_requests(
    client: &AdmissionControlClient,
    txn_requests: &[SubmitTransactionRequest],
) -> Vec<ProtoSubmitTransactionResponse> {
    let futures: Vec<_> = txn_requests
        .iter()
        .filter_map(|req| {
            match client.submit_transaction_async_opt(&req, get_default_grpc_call_option()) {
                Ok(future) => Some(future),
                Err(e) => {
                    OP_COUNTER.inc(&format!("submit_txns.{:?}", e));
                    error!("Failed to send gRPC request: {:?}", e);
                    None
                }
            }
        })
        .collect();
    // Wait all the futures unorderedly, then pick only accepted responses.
    stream::futures_unordered(futures)
        .wait()
        .filter_map(|future_result| match future_result {
            Ok(proto_resp) => {
                if check_ac_response(&proto_resp) {
                    Some(proto_resp)
                } else {
                    None
                }
            }
            Err(e) => {
                OP_COUNTER.inc(&format!("submit_txns.{:?}", e));
                error!("Failed to receive gRPC response: {:?}", e);
                None
            }
        })
        .collect()
}

/// ------------------------------------------------------------ ///
///  Account state async request and response handling helpers.  ///
/// ------------------------------------------------------------ ///

/// Send account state request async with a AC client.
fn get_account_state_async(
    client: &AdmissionControlClient,
    address: &AccountAddress,
) -> Result<impl Future<Item = UpdateToLatestLedgerResponse, Error = failure::Error>> {
    let requested_item = RequestItem::GetAccountState { address: *address };
    let requested_items = vec![requested_item];
    let req = UpdateToLatestLedgerRequest::new(0, requested_items);
    let proto_req = req.into_proto();
    let ret = client
        .update_to_latest_ledger_async_opt(&proto_req, get_default_grpc_call_option())?
        .then(move |account_state_proof_resp| {
            let resp = UpdateToLatestLedgerResponse::from_proto(account_state_proof_resp?)?;
            Ok(resp)
        });
    Ok(ret)
}

/// Wait and process the response of account state request.
/// For valid response, return account's sequence number and status.
fn handle_account_state_response(
    future_resp: impl Future<Item = UpdateToLatestLedgerResponse, Error = failure::Error>,
) -> Result<(u64, AccountStatus)> {
    let mut response = future_resp.wait()?;
    let account_state_proof = response
        .response_items
        .remove(0)
        .into_get_account_state_response()?;
    if let Some(account_state_blob) = account_state_proof.blob {
        let account_resource = get_account_resource_or_default(&Some(account_state_blob))?;
        Ok((account_resource.sequence_number(), AccountStatus::Persisted))
    } else {
        bail!("failed to get account state because account doesn't exist")
    }
}

/// Query a bunch of accounts' sequence numbers and status from an validator.
/// If operation for one account fails, during either querying or handling, return immediately.
/// When everything is fine, the return vector, whose item is (sequence_number, account_status),
/// is ensured to be in consistent order with the given account_address vector.
pub fn get_account_states(
    client: &AdmissionControlClient,
    account_address: &[AccountAddress],
) -> Result<Vec<(u64, AccountStatus)>> {
    let future_resps: Result<Vec<_>> = account_address
        .iter()
        .map(|address| get_account_state_async(client, address))
        .collect();
    future_resps?
        .into_iter()
        .map(handle_account_state_response)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::divide_items;

    #[test]
    fn test_divide_items() {
        let items: Vec<_> = (0..4).collect();
        let mut iter1 = divide_items(&items, 3);
        assert_eq!(iter1.next().unwrap(), &[0]);
        assert_eq!(iter1.next().unwrap(), &[1]);
        assert_eq!(iter1.next().unwrap(), &[2]);
        assert_eq!(iter1.next().unwrap(), &[3]);

        let mut iter2 = divide_items(&items, 2);
        assert_eq!(iter2.next().unwrap(), &[0, 1]);
        assert_eq!(iter2.next().unwrap(), &[2, 3]);

        let mut iter3 = divide_items(&items, 0);
        assert_eq!(iter3.next().unwrap(), &[0, 1, 2, 3]);

        let empty_slice: Vec<u32> = vec![];
        let mut empty_iter = divide_items(&empty_slice, 3);
        assert!(empty_iter.next().is_none());
        let mut empty_iter = divide_items(&empty_slice, 0);
        assert!(empty_iter.next().is_none());
    }
}
