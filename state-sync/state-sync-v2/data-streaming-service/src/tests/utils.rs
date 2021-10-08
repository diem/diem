// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::streaming_client::Epoch;
use async_trait::async_trait;
use diem_crypto::HashValue;
use diem_data_client::{
    AdvertisedData, DataClientPayload, DataClientResponse, DiemDataClient, GlobalDataSummary,
    OptimalChunkSizes, ResponseError,
};
use diem_types::{
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
};
use rand::{rngs::OsRng, RngCore};
use std::{collections::BTreeMap, thread, time::Duration};
use storage_service_types::CompleteDataRange;

/// Test constants for advertised data
pub const MIN_ADVERTISED_EPOCH: u64 = 100;
pub const MAX_ADVERTISED_EPOCH: u64 = 1000;

/// Test timeout constant
pub const MAX_NOTIFICATION_TIMEOUT_SECS: u64 = 4;

/// A simple mock of the Diem Data Client
#[derive(Clone)]
pub struct MockDiemDataClient {}

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
        // Sleep a random amount of time (< 1 second) to emulate network latencies
        thread::sleep(Duration::from_millis(create_random_u64(1000)));

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
            transaction_chunk_size: create_non_zero_random_u64(100),
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
            transactions: vec![],
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
        _start_version: u64,
        _end_version: u64,
        _include_events: bool,
    ) -> Result<DataClientResponse, diem_data_client::Error> {
        unimplemented!();
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
    let response_id = create_random_u64(10000);
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

/// Returns a random u64 with a value between 0 and `max_value` - 1 (inclusive).
pub fn create_random_u64(max_value: u64) -> u64 {
    let mut rng = OsRng;
    rng.next_u64() % max_value
}

/// Returns a random (but non-zero) u64 with a value between 1 and `max_value` - 1 (inclusive).
pub fn create_non_zero_random_u64(max_value: u64) -> u64 {
    create_random_u64(max_value - 1) + 1
}
