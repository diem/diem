// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::streaming_client::Epoch;
use async_trait::async_trait;
use diem_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, SigningKey, Uniform};
use diem_data_client::{
    AdvertisedData, DataClientPayload, DataClientResponse, DiemDataClient, GlobalDataSummary,
    OptimalChunkSizes, ResponseError,
};
use diem_types::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    chain_id::ChainId,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::{
        RawTransaction, Script, SignedTransaction, Transaction, TransactionListWithProof,
        TransactionPayload,
    },
};
use rand::{rngs::OsRng, RngCore};
use std::{collections::BTreeMap, thread, time::Duration};
use storage_service_types::CompleteDataRange;

/// Test constants for advertised data
pub const MAX_RESPONSE_ID: u64 = 100000;
pub const MIN_ADVERTISED_EPOCH: u64 = 100;
pub const MAX_ADVERTISED_EPOCH: u64 = 1000;
pub const MIN_ADVERTISED_TRANSACTION: u64 = 0;
pub const MAX_ADVERTISED_TRANSACTION: u64 = 10000;

/// Test timeout constant
pub const MAX_NOTIFICATION_TIMEOUT_SECS: u64 = 5;

/// A simple mock of the Diem Data Client
#[derive(Clone)]
pub struct MockDiemDataClient {}

impl MockDiemDataClient {
    fn emulate_network_latencies(&self) {
        // Sleep for 100 ms
        thread::sleep(Duration::from_millis(100));

        // Sleep an additional amount of time (< 0.4 second) to emulate variance
        thread::sleep(Duration::from_millis(create_random_u64(400)));
    }
}

#[async_trait]
impl DiemDataClient for MockDiemDataClient {
    async fn get_account_states_with_proof(
        &self,
        _version: u64,
        _start_index: HashValue,
        _end_index: HashValue,
    ) -> Result<DataClientResponse, diem_data_client::Error> {
        unimplemented!();
    }

    async fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<DataClientResponse, diem_data_client::Error> {
        self.emulate_network_latencies();

        // Create epoch ending ledger infos according to the requested epochs
        let mut epoch_ending_ledger_infos = vec![];
        for epoch in start_epoch..=end_epoch {
            epoch_ending_ledger_infos.push(create_ledger_info(epoch));
        }
        let response_payload = DataClientPayload::EpochEndingLedgerInfos(epoch_ending_ledger_infos);

        // Return the ledger infos
        Ok(create_data_client_response(response_payload))
    }

    fn get_global_data_summary(&self) -> Result<DataClientResponse, diem_data_client::Error> {
        // Create a random set of optimal chunk sizes to emulate changing environments
        let optimal_chunk_sizes = OptimalChunkSizes {
            account_states_chunk_size: create_non_zero_random_u64(100),
            epoch_chunk_size: create_non_zero_random_u64(100),
            transaction_chunk_size: create_non_zero_random_u64(2000),
            transaction_output_chunk_size: create_non_zero_random_u64(100),
        };

        // Create a global data summary with a fixed set of data
        let advertised_data = AdvertisedData {
            account_states: vec![],
            epoch_ending_ledger_infos: vec![CompleteDataRange::new(
                MIN_ADVERTISED_EPOCH,
                MAX_ADVERTISED_EPOCH,
            )],
            synced_ledger_infos: vec![],
            transactions: vec![CompleteDataRange::new(
                MIN_ADVERTISED_TRANSACTION,
                MAX_ADVERTISED_TRANSACTION,
            )],
            transaction_outputs: vec![],
        };
        let response_payload = DataClientPayload::GlobalDataSummary(GlobalDataSummary {
            advertised_data,
            optimal_chunk_sizes,
        });

        // Return the global data summary
        Ok(create_data_client_response(response_payload))
    }

    async fn get_number_of_account_states(
        &self,
        _version: u64,
    ) -> Result<DataClientResponse, diem_data_client::Error> {
        unimplemented!();
    }

    async fn get_transaction_outputs_with_proof(
        &self,
        _proof_version: u64,
        _start_version: u64,
        _end_version: u64,
    ) -> Result<DataClientResponse, diem_data_client::Error> {
        unimplemented!();
    }

    async fn get_transactions_with_proof(
        &self,
        _proof_version: u64,
        start_version: u64,
        end_version: u64,
        include_events: bool,
    ) -> Result<DataClientResponse, diem_data_client::Error> {
        self.emulate_network_latencies();

        // Include events if required
        let events = if include_events { Some(vec![]) } else { None };

        // Create the requested transactions
        let mut transactions = vec![];
        for _ in start_version..=end_version {
            transactions.push(create_transaction());
        }

        // Create a transaction list with an empty proof
        let mut transaction_list_with_proof = TransactionListWithProof::new_empty();
        transaction_list_with_proof.first_transaction_version = Some(start_version);
        transaction_list_with_proof.events = events;
        transaction_list_with_proof.transactions = transactions;
        let response_payload =
            DataClientPayload::TransactionsWithProof(transaction_list_with_proof);

        // Return the transaction list with proofs
        Ok(create_data_client_response(response_payload))
    }

    async fn notify_bad_response(
        &self,
        _response_id: u64,
        _response_error: ResponseError,
    ) -> Result<(), diem_data_client::Error> {
        unimplemented!();
    }
}

/// Creates a data client response using a specified payload and random id
pub fn create_data_client_response(response_payload: DataClientPayload) -> DataClientResponse {
    let response_id = create_random_u64(MAX_RESPONSE_ID);
    DataClientResponse {
        response_id,
        response_payload,
    }
}

/// Creates a ledger info with the given epoch
pub fn create_ledger_info(epoch: Epoch) -> LedgerInfoWithSignatures {
    let block_info = BlockInfo::new(epoch, 0, HashValue::zero(), HashValue::zero(), 0, 0, None);
    LedgerInfoWithSignatures::new(
        LedgerInfo::new(block_info, HashValue::zero()),
        BTreeMap::new(),
    )
}

/// Creates a simple test transaction
fn create_transaction() -> Transaction {
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let public_key = private_key.public_key();

    let transaction_payload = TransactionPayload::Script(Script::new(vec![], vec![], vec![]));
    let raw_transaction = RawTransaction::new(
        AccountAddress::random(),
        0,
        transaction_payload,
        0,
        0,
        "".into(),
        0,
        ChainId::new(10),
    );
    let signature = private_key.sign(&raw_transaction);
    let signed_transaction = SignedTransaction::new(raw_transaction, public_key, signature);

    Transaction::UserTransaction(signed_transaction)
}

/// Returns a random u64 with a value between 0 and `max_value` - 1 (inclusive).
fn create_random_u64(max_value: u64) -> u64 {
    let mut rng = OsRng;
    rng.next_u64() % max_value
}

/// Returns a random (but non-zero) u64 with a value between 1 and `max_value` - 1 (inclusive).
fn create_non_zero_random_u64(max_value: u64) -> u64 {
    create_random_u64(max_value - 1) + 1
}
