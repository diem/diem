// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides mock storage clients for tests.

use anyhow::{Error, Result};
use futures::prelude::*;
use libra_crypto::{ed25519::*, HashValue};
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_state_blob::AccountStateBlob,
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeEventWithProof},
    event::EventHandle,
    explorer::{
        GetTransactionByVersionResponse, GetTransactionListRequest, GetTransactionListResponse,
        LatestVersionResponse,
    },
    get_with_proof::{RequestItem, ResponseItem},
    proof::AccumulatorConsistencyProof,
    proof::SparseMerkleProof,
    proto::types::{
        request_item::RequestedItems, response_item::ResponseItems, AccountStateWithProof,
        GetAccountStateResponse, GetTransactionsResponse,
        LedgerInfoWithSignatures as ProtoLedgerInfoWithSignatures, RequestItem as ProtoRequestItem,
        ResponseItem as ProtoResponseItem, TransactionListWithProof, UpdateToLatestLedgerRequest,
        UpdateToLatestLedgerResponse,
    },
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{Transaction, Version},
    vm_error::StatusCode,
};
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use std::{collections::BTreeMap, convert::TryFrom, pin::Pin};
use storage_client::StorageRead;
use storage_proto::StartupInfo;

/// This is a mock of the storage read client used in tests.
///
/// See the real
/// [`StorageReadServiceClient`](../../../storage-client/struct.StorageReadServiceClient.html).
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
        ValidatorChangeEventWithProof,
        AccumulatorConsistencyProof,
    )> {
        let request = libra_types::get_with_proof::UpdateToLatestLedgerRequest::new(
            client_known_version,
            request_items,
        );
        let proto_request = request.into();
        let proto_response = get_mock_update_to_latest_ledger(&proto_request);
        let response =
            libra_types::get_with_proof::UpdateToLatestLedgerResponse::try_from(proto_response)?;
        Ok((
            response.response_items,
            response.ledger_info_with_sigs,
            response.validator_change_events,
            response.ledger_consistency_proof,
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
                        ValidatorChangeEventWithProof,
                        AccumulatorConsistencyProof,
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
    ) -> Result<libra_types::transaction::TransactionListWithProof> {
        unimplemented!()
    }

    fn get_transactions_async(
        &self,
        _start_version: Version,
        _batch_size: u64,
        _ledger_version: Version,
        _fetch_events: bool,
    ) -> Pin<
        Box<dyn Future<Output = Result<libra_types::transaction::TransactionListWithProof>> + Send>,
    > {
        unimplemented!()
    }

    fn get_account_state_with_proof_by_version(
        &self,
        _address: AccountAddress,
        _version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        unimplemented!()
    }

    fn get_account_state_with_proof_by_version_async(
        &self,
        _address: AccountAddress,
        _version: Version,
    ) -> Pin<Box<dyn Future<Output = Result<(Option<AccountStateBlob>, SparseMerkleProof)>> + Send>>
    {
        unimplemented!();
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        unimplemented!()
    }

    fn get_history_startup_info_by_block_id(
        &self,
        _block_id: HashValue,
    ) -> Result<Option<StartupInfo>> {
        unimplemented!()
    }

    fn get_startup_info_async(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<StartupInfo>>> + Send>> {
        unimplemented!()
    }

    fn get_epoch_change_ledger_infos(
        &self,
        _start_epoch: u64,
    ) -> Result<Vec<LedgerInfoWithSignatures>> {
        unimplemented!()
    }

    fn get_epoch_change_ledger_infos_async(
        &self,
        _start_epoch: u64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<LedgerInfoWithSignatures>>> + Send>> {
        unimplemented!()
    }

    fn latest_version(&self) -> Result<LatestVersionResponse> {
        unimplemented!()
    }

    fn get_transaction_list(
        &self,
        req: GetTransactionListRequest,
    ) -> Result<GetTransactionListResponse> {
        unimplemented!()
    }

    fn get_transaction_by_version(&self, req: Version) -> Result<GetTransactionByVersionResponse> {
        unimplemented!()
    }
}

fn get_mock_update_to_latest_ledger(
    req: &UpdateToLatestLedgerRequest,
) -> UpdateToLatestLedgerResponse {
    let mut resp = UpdateToLatestLedgerResponse::default();
    for request_item in req.requested_items.iter() {
        resp.response_items
            .push(get_mock_response_item(request_item).unwrap());
    }
    let mut ledger_info = libra_types::proto::types::LedgerInfo::default();
    ledger_info.transaction_accumulator_hash = HashValue::zero().to_vec();
    ledger_info.consensus_data_hash = HashValue::zero().to_vec();
    ledger_info.consensus_block_id = HashValue::zero().to_vec();
    ledger_info.version = 7;
    let mut ledger_info_with_sigs = ProtoLedgerInfoWithSignatures::default();
    ledger_info_with_sigs.ledger_info = Some(ledger_info);
    resp.ledger_info_with_sigs = Some(ledger_info_with_sigs);
    resp
}

fn get_mock_response_item(request_item: &ProtoRequestItem) -> Result<ProtoResponseItem> {
    let mut response_item = ProtoResponseItem::default();
    if let Some(ref requested_item) = request_item.requested_items {
        match requested_item {
            RequestedItems::GetAccountStateRequest(_request) => {
                let mut resp = GetAccountStateResponse::default();
                let mut version_data = BTreeMap::new();

                let account_resource = libra_types::account_config::AccountResource::new(
                    100,
                    0,
                    libra_types::byte_array::ByteArray::new(vec![]),
                    false,
                    false,
                    EventHandle::random_handle(0),
                    EventHandle::random_handle(0),
                    0,
                );
                version_data.insert(
                    libra_types::account_config::account_resource_path(),
                    lcs::to_bytes(&account_resource)?,
                );
                let mut account_state_with_proof = AccountStateWithProof::default();
                let blob = AccountStateBlob::from(lcs::to_bytes(&version_data)?).into();
                let proof = {
                    let ledger_info_to_transaction_info_proof =
                        libra_types::proof::AccumulatorProof::new(vec![]);
                    let transaction_info = libra_types::transaction::TransactionInfo::new(
                        HashValue::zero(),
                        HashValue::zero(),
                        HashValue::zero(),
                        0,
                        StatusCode::UNKNOWN_STATUS,
                    );
                    let transaction_info_to_account_proof =
                        libra_types::proof::SparseMerkleProof::new(None, vec![]);
                    libra_types::proof::AccountStateProof::new(
                        ledger_info_to_transaction_info_proof,
                        transaction_info,
                        transaction_info_to_account_proof,
                    )
                    .into()
                };
                account_state_with_proof.blob = Some(blob);
                account_state_with_proof.proof = Some(proof);
                resp.account_state_with_proof = Some(account_state_with_proof);
                response_item.response_items = Some(ResponseItems::GetAccountStateResponse(resp));
            }
            RequestedItems::GetAccountTransactionBySequenceNumberRequest(_request) => {
                unimplemented!();
            }
            RequestedItems::GetEventsByEventAccessPathRequest(_request) => {
                unimplemented!();
            }
            RequestedItems::GetTransactionsRequest(request) => {
                let mut ret = TransactionListWithProof::default();
                let sender = AccountAddress::new([1; ADDRESS_LENGTH]);
                if request.limit > 0 {
                    let txns = get_mock_txn_data(sender, 0, request.limit - 1);
                    ret.transactions = txns;
                }

                let mut resp = GetTransactionsResponse::default();
                resp.txn_list_with_proof = Some(ret);

                response_item.response_items = Some(ResponseItems::GetTransactionsResponse(resp));
            }
            RequestedItems::GetAccountStateByVersionRequest(_request) => {
                unimplemented!();
            }
        }
    }
    Ok(response_item)
}

fn get_mock_txn_data(
    address: AccountAddress,
    start_seq: u64,
    end_seq: u64,
) -> Vec<libra_types::proto::types::Transaction> {
    let mut seed_rng = OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng = StdRng::from_seed(seed_buf);
    let (priv_key, pub_key) = compat::generate_keypair(&mut rng);
    let mut txns = vec![];
    for i in start_seq..=end_seq {
        let txn = Transaction::UserTransaction(get_test_signed_txn(
            address,
            i,
            priv_key.clone(),
            pub_key.clone(),
            None,
        ));
        txns.push(txn.into());
    }
    txns
}
