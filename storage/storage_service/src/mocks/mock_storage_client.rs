// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides mock storage clients for tests.

use canonical_serialization::SimpleSerializer;
use crypto::{signing::generate_keypair, HashValue};
use failure::prelude::*;
use futures::prelude::*;
use proto_conv::{FromProto, IntoProto};
use std::{collections::BTreeMap, pin::Pin};
use storage_client::StorageRead;
use storage_proto::ExecutorStartupInfo;
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_state_blob::AccountStateBlob,
    get_with_proof::{RequestItem, ResponseItem},
    ledger_info::LedgerInfoWithSignatures,
    proof::definition::SparseMerkleProof,
    proto::{
        account_state_blob::AccountStateWithProof,
        get_with_proof::{
            GetAccountStateResponse, GetTransactionsResponse, RequestItem as ProtoRequestItem,
            RequestItem_oneof_requested_items, ResponseItem as ProtoResponseItem,
            UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
        },
        ledger_info::LedgerInfoWithSignatures as ProtoLedgerInfoWithSignatures,
        proof::AccumulatorProof,
        transaction::TransactionListWithProof,
        transaction_info::TransactionInfo,
    },
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::Version,
    validator_change::ValidatorChangeEventWithProof,
};

/// This is a mock of the storage read client used in tests.
///
/// See the real
/// [`StorageReadServiceClient`](../../../storage_client/struct.StorageReadServiceClient.html).
#[derive(Clone)]
pub struct MockStorageReadClient;

impl StorageRead for MockStorageReadClient {
    fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        Vec<ValidatorChangeEventWithProof>,
    )> {
        let request = types::get_with_proof::UpdateToLatestLedgerRequest::new(
            client_known_version,
            request_items,
        );
        let proto_request = request.into_proto();
        let proto_response = get_mock_update_to_latest_ledger(&proto_request);
        let response =
            types::get_with_proof::UpdateToLatestLedgerResponse::from_proto(proto_response)?;
        Ok((
            response.response_items,
            response.ledger_info_with_sigs,
            response.validator_change_events,
        ))
    }

    fn update_to_latest_ledger_async(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Pin<
        Box<
            dyn Future<
                    Output = Result<(
                        Vec<ResponseItem>,
                        LedgerInfoWithSignatures,
                        Vec<ValidatorChangeEventWithProof>,
                    )>,
                > + Send,
        >,
    > {
        futures::future::ok(
            self.update_to_latest_ledger(client_known_version, request_items)
                .unwrap(),
        )
        .boxed()
    }

    fn get_transactions(
        &self,
        _start_version: Version,
        _batch_size: u64,
        _ledger_version: Version,
        _fetch_events: bool,
    ) -> Result<types::transaction::TransactionListWithProof> {
        unimplemented!()
    }

    fn get_transactions_async(
        &self,
        _start_version: Version,
        _batch_size: u64,
        _ledger_version: Version,
        _fetch_events: bool,
    ) -> Pin<Box<dyn Future<Output = Result<types::transaction::TransactionListWithProof>> + Send>>
    {
        unimplemented!()
    }

    fn get_account_state_with_proof_by_state_root(
        &self,
        _address: AccountAddress,
        _state_root_hash: HashValue,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        unimplemented!()
    }

    fn get_account_state_with_proof_by_state_root_async(
        &self,
        _address: AccountAddress,
        _state_root_hash: HashValue,
    ) -> Pin<Box<dyn Future<Output = Result<(Option<AccountStateBlob>, SparseMerkleProof)>> + Send>>
    {
        unimplemented!();
    }

    fn get_executor_startup_info(&self) -> Result<Option<ExecutorStartupInfo>> {
        unimplemented!()
    }

    fn get_executor_startup_info_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<ExecutorStartupInfo>>> + Send>> {
        unimplemented!()
    }
}

fn get_mock_update_to_latest_ledger(
    req: &UpdateToLatestLedgerRequest,
) -> UpdateToLatestLedgerResponse {
    let mut resp = UpdateToLatestLedgerResponse::new();
    for request_item in req.get_requested_items().iter() {
        resp.mut_response_items()
            .push(get_mock_response_item(request_item).unwrap());
    }
    let mut ledger_info = types::proto::ledger_info::LedgerInfo::new();
    ledger_info.set_transaction_accumulator_hash(HashValue::zero().to_vec());
    ledger_info.set_consensus_data_hash(HashValue::zero().to_vec());
    ledger_info.set_consensus_block_id(HashValue::zero().to_vec());
    ledger_info.set_version(7);
    let mut ledger_info_with_sigs = ProtoLedgerInfoWithSignatures::new();
    ledger_info_with_sigs.set_ledger_info(ledger_info);
    resp.set_ledger_info_with_sigs(ledger_info_with_sigs);
    resp
}

fn get_mock_response_item(request_item: &ProtoRequestItem) -> Result<ProtoResponseItem> {
    let mut response_item = ProtoResponseItem::new();
    if let Some(ref requested_item) = request_item.requested_items {
        match requested_item {
            RequestItem_oneof_requested_items::get_account_state_request(_request) => {
                let mut resp = GetAccountStateResponse::new();
                let mut version_data = BTreeMap::new();

                let account_resource = types::account_config::AccountResource::new(
                    100,
                    0,
                    types::byte_array::ByteArray::new(vec![]),
                    0,
                    0,
                    false,
                );
                version_data.insert(
                    types::account_config::account_resource_path(),
                    SimpleSerializer::serialize(&account_resource)?,
                );
                let mut account_state_with_proof = AccountStateWithProof::new();
                let blob = AccountStateBlob::from(
                    SimpleSerializer::<Vec<u8>>::serialize(&version_data)?
                ).into_proto();
                let proof = {
                    let ledger_info_to_transaction_info_proof = types::proof::AccumulatorProof::new(vec![]);
                    let transaction_info = types::transaction::TransactionInfo::new(
                        HashValue::zero(),
                        HashValue::zero(),
                        HashValue::zero(),
                        0,
                    );
                    let transaction_info_to_account_proof = types::proof::SparseMerkleProof::new(None, vec![]);
                    types::proof::AccountStateProof::new(
                        ledger_info_to_transaction_info_proof,
                        transaction_info,
                        transaction_info_to_account_proof,
                    ).into_proto()
                };
                account_state_with_proof.set_blob(blob);
                account_state_with_proof.set_proof(proof);
                resp.set_account_state_with_proof(account_state_with_proof);
                response_item.set_get_account_state_response(resp);
            }
            RequestItem_oneof_requested_items::get_account_transaction_by_sequence_number_request(_request) => {
                unimplemented!();
            }
            RequestItem_oneof_requested_items::get_events_by_event_access_path_request(_request) => {
                unimplemented!();
            }
            RequestItem_oneof_requested_items::get_transactions_request(request) => {
                let mut ret = TransactionListWithProof::new();
                let sender = AccountAddress::new([1; ADDRESS_LENGTH]);
                if request.limit > 0 {
                    let (txns, infos) = get_mock_txn_data(sender, 0, request.limit - 1);
                    if !txns.is_empty() {
                        ret.set_proof_of_first_transaction(get_accumulator_proof());
                    }
                    if txns.len() >= 2 {
                        ret.set_proof_of_last_transaction(get_accumulator_proof());
                    }
                    ret.set_transactions(protobuf::RepeatedField::from_vec(txns));
                    ret.set_infos(protobuf::RepeatedField::from_vec(infos));
                }

                let mut resp = GetTransactionsResponse::new();
                resp.set_txn_list_with_proof(ret);

                response_item.set_get_transactions_response(resp);
            }
        }
    }
    Ok(response_item)
}

fn get_mock_txn_data(
    address: AccountAddress,
    start_seq: u64,
    end_seq: u64,
) -> (
    Vec<types::proto::transaction::SignedTransaction>,
    Vec<TransactionInfo>,
) {
    let (priv_key, pub_key) = generate_keypair();
    let mut txns = vec![];
    let mut infos = vec![];
    for i in start_seq..=end_seq {
        let signed_txn = get_test_signed_txn(address, i, priv_key.clone(), pub_key, None);
        txns.push(signed_txn);

        let info = get_transaction_info().into_proto();
        infos.push(info);
    }
    (txns, infos)
}

fn get_accumulator_proof() -> AccumulatorProof {
    types::proof::AccumulatorProof::new(vec![]).into_proto()
}

fn get_transaction_info() -> types::transaction::TransactionInfo {
    types::transaction::TransactionInfo::new(
        HashValue::zero(),
        HashValue::zero(),
        HashValue::zero(),
        0,
    )
}
