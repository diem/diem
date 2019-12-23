// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use futures::stream::StreamExt;
use grpcio::EnvBuilder;
use itertools::zip_eq;
use libra_config::config::NodeConfig;
use libra_crypto::hash::CryptoHash;
use libra_types::get_with_proof::{RequestItem, ResponseItem};
#[cfg(test)]
use libradb::test_helper::arb_blocks_to_commit;
use proptest::prelude::*;
use std::collections::{BTreeMap, HashMap};
use storage_client::{
    StorageRead, StorageReadServiceClient, StorageWrite, StorageWriteServiceClient,
};
use tokio::runtime::Runtime;

fn start_test_storage_with_read_write_client() -> (
    Runtime,
    libra_tools::tempdir::TempPath,
    ServerHandle,
    StorageReadServiceClient,
    StorageWriteServiceClient,
) {
    let rt = Runtime::new().unwrap();
    let mut config = NodeConfig::random();
    let tmp_dir = libra_tools::tempdir::TempPath::new();
    config.storage.dir = tmp_dir.path().to_path_buf();

    let storage_server_handle = start_storage_service(&config);

    let read_client = StorageReadServiceClient::new(
        Arc::new(EnvBuilder::new().build()),
        &config.storage.address,
        config.storage.port,
    );
    let write_client = StorageWriteServiceClient::new(
        Arc::new(EnvBuilder::new().build()),
        &config.storage.address,
        config.storage.port,
        None,
    );
    (
        rt,
        tmp_dir,
        storage_server_handle,
        read_client,
        write_client,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_storage_service_basic(blocks in arb_blocks_to_commit().no_shrink()) {
        let(mut rt, _tmp_dir, _server_handler, read_client, write_client) =
            start_test_storage_with_read_write_client();

        let mut version = 0;
        let mut all_accounts = BTreeMap::new();

        for (txns_to_commit, ledger_info_with_sigs) in &blocks {
            rt.block_on(write_client
                .save_transactions_async(txns_to_commit.clone(),
                                   version, /* first_version */
                                   Some(ledger_info_with_sigs.clone()),
                )).unwrap();
            version += txns_to_commit.len() as u64;
            let mut account_states = HashMap::new();
            // Get the ground truth of account states.
            txns_to_commit
                .iter()
                .for_each(|txn_to_commit|
                          account_states.extend(txn_to_commit
                                                .account_states()
                                                .clone())
                );

            // Record all account states.
            for (address, blob) in account_states.iter() {
                all_accounts.insert(address.hash(), blob.clone());
            }

            let account_state_request_items = account_states
                .keys()
                .map(|address| RequestItem::GetAccountState{
                    address: *address,
                }).collect::<Vec<_>>();
            let (
                response_items,
                response_ledger_info_with_sigs,
                _validator_change_proof,
                _ledger_consistency_proof,
            ) = rt.block_on(read_client
                .update_to_latest_ledger_async(0, account_state_request_items)).unwrap();
            for ((address, blob), response_item) in zip_eq(account_states, response_items) {
                    match response_item {
                        ResponseItem::GetAccountState {
                            account_state_with_proof,
                        } => {
                            prop_assert_eq!(&Some(blob), &account_state_with_proof.blob);
                            prop_assert!(account_state_with_proof.verify(
                                response_ledger_info_with_sigs.ledger_info(),
                                version - 1,
                                address,
                            ).is_ok())
                        }
                        _ => unreachable!()
                    }
            }

            // Assert ledger info.
            prop_assert_eq!(ledger_info_with_sigs, &response_ledger_info_with_sigs);
        }

        // Check state backup for all account states.
        let stream = read_client.backup_account_state(version - 1).unwrap();
        let backup_responses = rt.block_on(stream.collect::<Vec<_>>());
        for ((hash, blob), response) in zip_eq(all_accounts, backup_responses) {
            let resp = response.unwrap();
            prop_assert_eq!(&hash, &resp.account_key);
            prop_assert_eq!(&blob, &resp.account_state_blob);
        }
    }
}
