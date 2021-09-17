// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{StorageReader, StorageServiceServer};
use anyhow::Result;
use claim::{assert_none, assert_some};
use diem_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, SigningKey, Uniform};
use diem_infallible::RwLock;
use diem_types::{
    account_address::AccountAddress,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    block_info::BlockInfo,
    chain_id::ChainId,
    contract_event::{ContractEvent, EventByVersionWithProof, EventWithProof},
    epoch_change::EpochChangeProof,
    event::EventKey,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::{SparseMerkleProof, TransactionInfoListWithProof},
    state_proof::StateProof,
    transaction::{
        AccountTransactionsWithProof, RawTransaction, Script, SignedTransaction, Transaction,
        TransactionListWithProof, TransactionPayload, TransactionToCommit, TransactionWithProof,
        Version,
    },
};
use move_core_types::language_storage::TypeTag;
use std::{collections::BTreeMap, sync::Arc};
use storage_interface::{DbReader, DbReaderWriter, DbWriter, Order, StartupInfo, TreeState};
use storage_service_types::{
    DataSummary, EpochEndingLedgerInfoRequest, ProtocolMetadata, ServerProtocolVersion,
    StorageServerSummary, StorageServiceRequest, StorageServiceResponse,
    TransactionsWithProofRequest,
};

// TODO(joshlind): Expand these test cases to better test storage interaction
// and functionality. This will likely require a better mock db abstraction.

#[test]
fn test_get_server_protocol_version() {
    // Create a storage service server
    let storage_server = create_storage_server();

    // Process a request to fetch the protocol version
    let version_request = StorageServiceRequest::GetServerProtocolVersion;
    let version_response = storage_server.handle_request(version_request).unwrap();

    // Verify the response is correct
    let expected_protocol_version = ServerProtocolVersion {
        protocol_version: 1,
    };
    assert_eq!(
        version_response,
        StorageServiceResponse::ServerProtocolVersion(expected_protocol_version)
    );
}

#[test]
fn test_get_storage_server_summary() {
    // Create a storage service server
    let storage_server = create_storage_server();

    // Process a request to fetch the storage summary
    let summary_request = StorageServiceRequest::GetStorageServerSummary;
    let summary_response = storage_server.handle_request(summary_request).unwrap();

    // Verify the response is correct
    let expected_server_summary = StorageServerSummary {
        protocol_metadata: ProtocolMetadata {
            max_transaction_chunk_size: 1000,
        },
        data_summary: DataSummary {
            highest_transaction_version: 100,
            lowest_transaction_version: 0,
            highest_epoch: 10,
            lowest_epoch: 0,
        },
    };
    assert_eq!(
        summary_response,
        StorageServiceResponse::StorageServerSummary(expected_server_summary)
    );
}

#[test]
fn test_get_transactions_with_proof_events() {
    // Create a storage service server
    let storage_server = create_storage_server();

    // Create a request to fetch transactions with a proof
    let start_version = 0;
    let expected_num_transactions = 10;
    let transactions_proof_request =
        StorageServiceRequest::GetTransactionsWithProof(TransactionsWithProofRequest {
            proof_version: 100,
            start_version,
            expected_num_transactions,
            include_events: true,
        });

    // Process the request
    let transactions_proof_response = storage_server
        .handle_request(transactions_proof_request)
        .unwrap();

    // Verify the response is correct
    match transactions_proof_response {
        StorageServiceResponse::TransactionsWithProof(transactions_with_proof) => {
            assert_eq!(
                transactions_with_proof.transactions.len(),
                expected_num_transactions as usize
            );
            assert_eq!(
                transactions_with_proof.first_transaction_version,
                Some(start_version)
            );
            assert_some!(transactions_with_proof.events);
        }
        result => {
            panic!("Expected transactions with proof but got: {:?}", result);
        }
    };
}

#[test]
fn test_get_transactions_with_proof_no_events() {
    // Create a storage service server
    let storage_server = create_storage_server();

    // Create a request to fetch transactions with a proof (excluding events)
    let start_version = 10;
    let expected_num_transactions = 20;
    let transactions_proof_request =
        StorageServiceRequest::GetTransactionsWithProof(TransactionsWithProofRequest {
            proof_version: 1000,
            start_version,
            expected_num_transactions,
            include_events: false,
        });

    // Process the request
    let transactions_proof_response = storage_server
        .handle_request(transactions_proof_request)
        .unwrap();

    // Verify the response is correct
    match transactions_proof_response {
        StorageServiceResponse::TransactionsWithProof(transactions_with_proof) => {
            assert_eq!(
                transactions_with_proof.transactions.len(),
                expected_num_transactions as usize
            );
            assert_eq!(
                transactions_with_proof.first_transaction_version,
                Some(start_version)
            );
            assert_none!(transactions_with_proof.events);
        }
        result => {
            panic!("Expected transactions with proof but got: {:?}", result);
        }
    };
}

#[test]
fn test_get_epoch_ending_ledger_infos() {
    // Create a storage service server
    let storage_server = create_storage_server();

    // Create a request to fetch transactions with a proof (excluding events)
    let start_epoch = 11;
    let expected_end_epoch = 21;
    let epoch_ending_li_request =
        StorageServiceRequest::GetEpochEndingLedgerInfos(EpochEndingLedgerInfoRequest {
            start_epoch,
            expected_end_epoch,
        });

    // Process the request
    let epoch_ending_li_response = storage_server
        .handle_request(epoch_ending_li_request)
        .unwrap();

    // Verify the response is correct
    match epoch_ending_li_response {
        StorageServiceResponse::EpochEndingLedgerInfos(epoch_change_proof) => {
            assert_eq!(
                epoch_change_proof.ledger_info_with_sigs.len(),
                (expected_end_epoch - start_epoch + 1) as usize
            );
            assert_eq!(epoch_change_proof.more, false);

            for (i, epoch_ending_li) in epoch_change_proof.ledger_info_with_sigs.iter().enumerate()
            {
                assert_eq!(
                    epoch_ending_li.ledger_info().epoch(),
                    (i as u64) + start_epoch
                );
            }
        }
        result => {
            panic!("Expected epoch ending ledger infos but got: {:?}", result);
        }
    };
}

fn create_storage_server() -> StorageServiceServer<StorageReader> {
    let storage = Arc::new(RwLock::new(DbReaderWriter::new(MockDbReaderWriter)));
    let storage_reader = StorageReader::new(storage);
    StorageServiceServer::new(storage_reader)
}

fn create_test_event(sequence_number: u64) -> ContractEvent {
    ContractEvent::new(
        EventKey::new_from_address(&AccountAddress::random(), 0),
        sequence_number,
        TypeTag::Bool,
        bcs::to_bytes(&0).unwrap(),
    )
}

fn create_test_transaction(sequence_number: u64) -> Transaction {
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let public_key = private_key.public_key();

    let transaction_payload = TransactionPayload::Script(Script::new(vec![], vec![], vec![]));
    let raw_transaction = RawTransaction::new(
        AccountAddress::random(),
        sequence_number,
        transaction_payload,
        0,
        0,
        "".into(),
        0,
        ChainId::new(10),
    );
    let signed_transaction = SignedTransaction::new(
        raw_transaction.clone(),
        public_key,
        private_key.sign(&raw_transaction),
    );

    Transaction::UserTransaction(signed_transaction)
}

fn create_test_ledger_info_with_sigs(epoch: u64, version: u64) -> LedgerInfoWithSignatures {
    // Create a mock ledger info with signatures
    let ledger_info = LedgerInfo::new(
        BlockInfo::new(
            epoch,
            0,
            HashValue::zero(),
            HashValue::zero(),
            version,
            0,
            None,
        ),
        HashValue::zero(),
    );
    LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new())
}

/// This is a mock of the DbReader and DbWriter for unit testing.
struct MockDbReaderWriter;

impl DbReader for MockDbReaderWriter {
    fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<EpochChangeProof> {
        let mut ledger_info_with_sigs = vec![];
        for epoch in start_epoch..end_epoch + 1 {
            ledger_info_with_sigs.push(create_test_ledger_info_with_sigs(epoch, 0));
        }

        Ok(EpochChangeProof {
            ledger_info_with_sigs,
            more: false,
        })
    }

    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        _ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        // Create mock events
        let events = if fetch_events {
            let mut events = vec![];
            for i in 0..batch_size {
                events.push(vec![create_test_event(i)]);
            }
            Some(events)
        } else {
            None
        };

        // Create mock transactions
        let mut transactions = vec![];
        for i in 0..batch_size {
            transactions.push(create_test_transaction(i))
        }

        Ok(TransactionListWithProof {
            transactions,
            events,
            first_transaction_version: Some(start_version),
            proof: TransactionInfoListWithProof::new_empty(),
        })
    }

    /// Returns events by given event key
    fn get_events(
        &self,
        _event_key: &EventKey,
        _start: u64,
        _order: Order,
        _limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>> {
        unimplemented!()
    }

    /// Returns events by given event key
    fn get_events_with_proofs(
        &self,
        _event_key: &EventKey,
        _start: u64,
        _order: Order,
        _limit: u64,
        _known_version: Option<u64>,
    ) -> Result<Vec<EventWithProof>> {
        unimplemented!()
    }

    fn get_block_timestamp(&self, _version: u64) -> Result<u64> {
        unimplemented!()
    }

    fn get_event_by_version_with_proof(
        &self,
        _event_key: &EventKey,
        _version: u64,
        _proof_version: u64,
    ) -> Result<EventByVersionWithProof> {
        unimplemented!()
    }

    fn get_latest_account_state(
        &self,
        _address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        unimplemented!()
    }

    /// Returns the latest ledger info.
    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        Ok(create_test_ledger_info_with_sigs(10, 100))
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        unimplemented!()
    }

    fn get_account_transaction(
        &self,
        _address: AccountAddress,
        _seq_num: u64,
        _include_events: bool,
        _ledger_version: Version,
    ) -> Result<Option<TransactionWithProof>> {
        unimplemented!()
    }

    fn get_account_transactions(
        &self,
        _address: AccountAddress,
        _start_seq_num: u64,
        _limit: u64,
        _include_events: bool,
        _ledger_version: Version,
    ) -> Result<AccountTransactionsWithProof> {
        unimplemented!()
    }

    fn get_state_proof_with_ledger_info(
        &self,
        _known_version: u64,
        _ledger_info: LedgerInfoWithSignatures,
    ) -> Result<StateProof> {
        unimplemented!()
    }

    fn get_state_proof(&self, _known_version: u64) -> Result<StateProof> {
        unimplemented!()
    }

    fn get_account_state_with_proof(
        &self,
        _address: AccountAddress,
        _version: Version,
        _ledger_version: Version,
    ) -> Result<AccountStateWithProof> {
        unimplemented!()
    }

    fn get_account_state_with_proof_by_version(
        &self,
        _address: AccountAddress,
        _version: Version,
    ) -> Result<(
        Option<AccountStateBlob>,
        SparseMerkleProof<AccountStateBlob>,
    )> {
        unimplemented!()
    }

    fn get_latest_state_root(&self) -> Result<(Version, HashValue)> {
        unimplemented!()
    }

    fn get_latest_tree_state(&self) -> Result<TreeState> {
        unimplemented!()
    }

    fn get_epoch_ending_ledger_info(
        &self,
        _known_version: u64,
    ) -> Result<LedgerInfoWithSignatures> {
        unimplemented!()
    }
}

impl DbWriter for MockDbReaderWriter {
    fn save_transactions(
        &self,
        _txns_to_commit: &[TransactionToCommit],
        _first_version: Version,
        _ledger_info_with_sigs: Option<&LedgerInfoWithSignatures>,
    ) -> Result<()> {
        unimplemented!()
    }
}
