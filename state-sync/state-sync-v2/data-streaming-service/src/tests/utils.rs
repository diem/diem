// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{data_notification::DataNotification, data_stream::DataStreamListener, error::Error};
use async_trait::async_trait;
use diem_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, SigningKey, Uniform};
use diem_data_client::{
    AdvertisedData, DiemDataClient, GlobalDataSummary, OptimalChunkSizes, Response,
    ResponseCallback, ResponseContext, ResponseError,
};
use diem_logger::Level;
use diem_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStatesChunkWithProof,
    block_info::BlockInfo,
    chain_id::ChainId,
    epoch_state::EpochState,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::SparseMerkleRangeProof,
    transaction::{
        RawTransaction, Script, SignedTransaction, Transaction, TransactionListWithProof,
        TransactionOutput, TransactionOutputListWithProof, TransactionPayload, TransactionStatus,
        Version,
    },
    write_set::WriteSet,
};
use futures::StreamExt;
use rand::{rngs::OsRng, Rng};
use std::{
    collections::{BTreeMap, HashMap},
    thread,
    time::Duration,
};
use storage_service_types::{CompleteDataRange, Epoch};
use tokio::time::timeout;

/// The number of accounts held at any version
pub const TOTAL_NUM_ACCOUNTS: u64 = 2000;

/// Test constants for advertised data
pub const MAX_RESPONSE_ID: u64 = 100000;
pub const MIN_ADVERTISED_ACCOUNTS: u64 = 9500;
pub const MAX_ADVERTISED_ACCOUNTS: u64 = 10000;
pub const MIN_ADVERTISED_EPOCH_END: u64 = 100;
pub const MAX_ADVERTISED_EPOCH_END: u64 = 150;
pub const MIN_ADVERTISED_TRANSACTION: u64 = 1000;
pub const MAX_ADVERTISED_TRANSACTION: u64 = 10000;
pub const MIN_ADVERTISED_TRANSACTION_OUTPUT: u64 = 1000;
pub const MAX_ADVERTISED_TRANSACTION_OUTPUT: u64 = 10000;

/// Test timeout constant
pub const MAX_NOTIFICATION_TIMEOUT_SECS: u64 = 10;

/// A simple mock of the Diem Data Client
#[derive(Clone, Debug)]
pub struct MockDiemDataClient {
    pub epoch_ending_ledger_infos: HashMap<Epoch, LedgerInfoWithSignatures>,
    pub synced_ledger_infos: Vec<LedgerInfoWithSignatures>,
}

impl MockDiemDataClient {
    pub fn new() -> Self {
        let epoch_ending_ledger_infos = create_epoch_ending_ledger_infos();
        let synced_ledger_infos = create_synced_ledger_infos(&epoch_ending_ledger_infos);

        Self {
            epoch_ending_ledger_infos,
            synced_ledger_infos,
        }
    }

    fn emulate_network_latencies(&self) {
        // Sleep for 100 - 500 ms to emulate variance
        thread::sleep(Duration::from_millis(create_range_random_u64(100, 500)));
    }
}

#[async_trait]
impl DiemDataClient for MockDiemDataClient {
    fn get_global_data_summary(&self) -> GlobalDataSummary {
        // Create a random set of optimal chunk sizes to emulate changing environments
        let optimal_chunk_sizes = OptimalChunkSizes {
            account_states_chunk_size: create_non_zero_random_u64(100),
            epoch_chunk_size: create_non_zero_random_u64(10),
            transaction_chunk_size: create_non_zero_random_u64(2000),
            transaction_output_chunk_size: create_non_zero_random_u64(100),
        };

        // Create a global data summary with a fixed set of data
        let advertised_data = AdvertisedData {
            account_states: vec![CompleteDataRange::new(
                MIN_ADVERTISED_ACCOUNTS,
                MAX_ADVERTISED_ACCOUNTS,
            )
            .unwrap()],
            epoch_ending_ledger_infos: vec![CompleteDataRange::new(
                MIN_ADVERTISED_EPOCH_END,
                MAX_ADVERTISED_EPOCH_END,
            )
            .unwrap()],
            synced_ledger_infos: self.synced_ledger_infos.clone(),
            transactions: vec![CompleteDataRange::new(
                MIN_ADVERTISED_TRANSACTION,
                MAX_ADVERTISED_TRANSACTION,
            )
            .unwrap()],
            transaction_outputs: vec![CompleteDataRange::new(
                MIN_ADVERTISED_TRANSACTION_OUTPUT,
                MAX_ADVERTISED_TRANSACTION_OUTPUT,
            )
            .unwrap()],
        };
        GlobalDataSummary {
            advertised_data,
            optimal_chunk_sizes,
        }
    }

    async fn get_account_states_with_proof(
        &self,
        _version: Version,
        start_index: u64,
        end_index: u64,
    ) -> Result<Response<AccountStatesChunkWithProof>, diem_data_client::Error> {
        self.emulate_network_latencies();

        // Create epoch ending ledger infos according to the requested epochs
        let mut account_blobs = vec![];
        for _ in start_index..=end_index {
            account_blobs.push((HashValue::random(), vec![].into()));
        }

        // Create an account states chunk with proof
        let account_states = AccountStatesChunkWithProof {
            first_index: start_index,
            last_index: end_index,
            first_key: HashValue::random(),
            last_key: HashValue::random(),
            account_blobs,
            proof: SparseMerkleRangeProof::new(vec![]),
            root_hash: HashValue::zero(),
        };
        Ok(create_data_client_response(account_states))
    }

    async fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: Epoch,
        end_epoch: Epoch,
    ) -> Result<Response<Vec<LedgerInfoWithSignatures>>, diem_data_client::Error> {
        self.emulate_network_latencies();

        // Fetch the epoch ending ledger infos according to the requested epochs
        let mut epoch_ending_ledger_infos = vec![];
        for epoch in start_epoch..=end_epoch {
            let ledger_info = self.epoch_ending_ledger_infos.get(&epoch).unwrap();
            epoch_ending_ledger_infos.push(ledger_info.clone());
        }
        Ok(create_data_client_response(epoch_ending_ledger_infos))
    }

    async fn get_number_of_account_states(
        &self,
        _version: Version,
    ) -> Result<Response<u64>, diem_data_client::Error> {
        Ok(create_data_client_response(TOTAL_NUM_ACCOUNTS))
    }

    async fn get_transaction_outputs_with_proof(
        &self,
        _proof_version: Version,
        start_version: Version,
        end_version: Version,
    ) -> Result<Response<TransactionOutputListWithProof>, diem_data_client::Error> {
        self.emulate_network_latencies();

        // Create the requested transactions and transaction outputs
        let mut transactions_and_outputs = vec![];
        for _ in start_version..=end_version {
            transactions_and_outputs.push((create_transaction(), create_transaction_output()));
        }

        // Create a transaction output list with an empty proof
        let mut output_list_with_proof = TransactionOutputListWithProof::new_empty();
        output_list_with_proof.first_transaction_output_version = Some(start_version);
        output_list_with_proof.transactions_and_outputs = transactions_and_outputs;
        Ok(create_data_client_response(output_list_with_proof))
    }

    async fn get_transactions_with_proof(
        &self,
        _proof_version: Version,
        start_version: Version,
        end_version: Version,
        include_events: bool,
    ) -> Result<Response<TransactionListWithProof>, diem_data_client::Error> {
        self.emulate_network_latencies();

        let transaction_list_with_proof =
            create_transaction_list_with_proof(start_version, end_version, include_events);

        // Return the transaction list with proofs
        Ok(create_data_client_response(transaction_list_with_proof))
    }
}

#[derive(Debug)]
pub struct NoopResponseCallback;

impl ResponseCallback for NoopResponseCallback {
    fn notify_bad_response(&self, _error: ResponseError) {
        // TODO(philiphayes): do something here
    }
}

/// Creates a data client response using a specified payload and random id
pub fn create_data_client_response<T>(payload: T) -> Response<T> {
    let id = create_random_u64(MAX_RESPONSE_ID);
    let response_callback = Box::new(NoopResponseCallback);
    let context = ResponseContext {
        id,
        response_callback,
    };
    Response::new(context, payload)
}

/// Creates a ledger info with the given version and epoch. If `epoch_ending`
/// is true, makes the ledger info an epoch ending ledger info.
pub fn create_ledger_info(
    version: Version,
    epoch: Epoch,
    epoch_ending: bool,
) -> LedgerInfoWithSignatures {
    let next_epoch_state = if epoch_ending {
        let mut epoch_state = EpochState::empty();
        epoch_state.epoch = epoch + 1;
        Some(epoch_state)
    } else {
        None
    };

    let block_info = BlockInfo::new(
        epoch,
        0,
        HashValue::zero(),
        HashValue::zero(),
        version,
        0,
        next_epoch_state,
    );
    LedgerInfoWithSignatures::new(
        LedgerInfo::new(block_info, HashValue::zero()),
        BTreeMap::new(),
    )
}

/// Creates a epoch ending ledger infos for all epochs
fn create_epoch_ending_ledger_infos() -> HashMap<Epoch, LedgerInfoWithSignatures> {
    let mut current_epoch = MIN_ADVERTISED_EPOCH_END;
    let mut current_version = MIN_ADVERTISED_TRANSACTION;

    // Populate the epoch ending ledger infos using random intervals
    let max_num_versions_in_epoch = (MAX_ADVERTISED_TRANSACTION - MIN_ADVERTISED_TRANSACTION)
        / ((MAX_ADVERTISED_EPOCH_END + 1) - MIN_ADVERTISED_EPOCH_END);
    let mut epoch_ending_ledger_infos = HashMap::new();
    while current_epoch < MAX_ADVERTISED_EPOCH_END + 1 {
        let num_versions_in_epoch = create_non_zero_random_u64(max_num_versions_in_epoch);
        current_version += num_versions_in_epoch;

        if epoch_ending_ledger_infos
            .insert(
                current_epoch,
                create_ledger_info(current_version, current_epoch, true),
            )
            .is_some()
        {
            panic!("Duplicate epoch ending ledger info found! This should not occur!",);
        }
        current_epoch += 1;
    }

    epoch_ending_ledger_infos
}

/// Creates a set of synced ledger infos for advertising
fn create_synced_ledger_infos(
    epoch_ending_ledger_infos: &HashMap<Epoch, LedgerInfoWithSignatures>,
) -> Vec<LedgerInfoWithSignatures> {
    let mut current_epoch = MIN_ADVERTISED_EPOCH_END;
    let mut current_version = MIN_ADVERTISED_TRANSACTION;

    // Populate the synced ledger infos
    let mut synced_ledger_infos = vec![];
    while current_version < MAX_ADVERTISED_TRANSACTION && current_epoch < MAX_ADVERTISED_EPOCH_END {
        let random_num_versions = create_non_zero_random_u64(10);
        current_version += random_num_versions;

        let end_of_epoch_version = epoch_ending_ledger_infos
            .get(&current_epoch)
            .unwrap()
            .ledger_info()
            .version();
        if current_version > end_of_epoch_version {
            current_epoch += 1;
        }

        let end_of_epoch = end_of_epoch_version == current_version;
        synced_ledger_infos.push(create_ledger_info(
            current_version,
            current_epoch,
            end_of_epoch,
        ));
    }

    // Manually insert a synced ledger info at the last transaction and epoch
    // to ensure we can sync right up to the end.
    synced_ledger_infos.push(create_ledger_info(
        MAX_ADVERTISED_TRANSACTION,
        MAX_ADVERTISED_EPOCH_END,
        false,
    ));

    synced_ledger_infos
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

/// Creates an empty transaction output
fn create_transaction_output() -> TransactionOutput {
    TransactionOutput::new(WriteSet::default(), vec![], 0, TransactionStatus::Retry)
}

/// Returns a random u64 with a value between 0 and `max_value` - 1 (inclusive).
pub fn create_random_u64(max_value: u64) -> u64 {
    create_range_random_u64(0, max_value)
}

/// Returns a random (but non-zero) u64 with a value between 1 and `max_value` - 1 (inclusive).
fn create_non_zero_random_u64(max_value: u64) -> u64 {
    create_range_random_u64(1, max_value)
}

/// Returns a random u64 within the range, [min, max)
fn create_range_random_u64(min_value: u64, max_value: u64) -> u64 {
    let mut rng = OsRng;
    rng.gen_range(min_value..max_value)
}

/// Initializes the Diem logger for tests
pub fn initialize_logger() {
    diem_logger::DiemLogger::builder()
        .is_async(false)
        .level(Level::Info)
        .build();
}

/// Returns a data notification from the given stream listener
pub async fn get_data_notification(
    stream_listener: &mut DataStreamListener,
) -> Result<DataNotification, Error> {
    if let Ok(data_notification) = timeout(
        Duration::from_secs(MAX_NOTIFICATION_TIMEOUT_SECS),
        stream_listener.select_next_some(),
    )
    .await
    {
        Ok(data_notification)
    } else {
        Err(Error::UnexpectedErrorEncountered(
            "Timed out waiting for a data notification!".into(),
        ))
    }
}

pub fn create_transaction_list_with_proof(
    start_version: u64,
    end_version: u64,
    include_events: bool,
) -> TransactionListWithProof {
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

    transaction_list_with_proof
}
