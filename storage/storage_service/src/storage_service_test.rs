// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use grpcio::EnvBuilder;
use itertools::zip_eq;
use libra_config::config::NodeConfigHelpers;
use libra_storage_client::{
    StorageRead, StorageReadServiceClient, StorageWrite, StorageWriteServiceClient,
};
use libra_types::get_with_proof::{RequestItem, ResponseItem};
use libradb::mock_genesis::db_with_mock_genesis;
#[cfg(any(test, feature = "testing"))]
use libradb::test_helper::arb_blocks_to_commit;
use proptest::prelude::*;
use std::collections::HashMap;

fn start_test_storage_with_read_write_client(
    need_to_use_genesis: bool,
) -> (
    libra_tools::tempdir::TempPath,
    ServerHandle,
    StorageReadServiceClient,
    StorageWriteServiceClient,
) {
    let mut config = NodeConfigHelpers::get_single_node_test_config(/* random_ports = */ true);
    let tmp_dir = libra_tools::tempdir::TempPath::new();
    config.storage.dir = tmp_dir.path().to_path_buf();

    // initialize db with genesis info.
    if need_to_use_genesis {
        db_with_mock_genesis(&tmp_dir).unwrap();
    } else {
        LibraDB::new(&tmp_dir);
    }
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
    (tmp_dir, storage_server_handle, read_client, write_client)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_storage_service_basic(blocks in arb_blocks_to_commit().no_shrink()) {
        let(_tmp_dir, _server_handler, read_client, write_client) =
            start_test_storage_with_read_write_client(/* need_to_use_genesis = */ true);

        let mut version = 0;
        for (txns_to_commit, ledger_info_with_sigs) in &blocks {
            write_client
                .save_transactions(txns_to_commit.clone(),
                                   version + 1, /* first_version */
                                   Some(ledger_info_with_sigs.clone()),
                ).unwrap();
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

            let account_state_request_items = account_states
                .keys()
                .map(|address| RequestItem::GetAccountState{
                    address: *address,
                }).collect::<Vec<_>>();
            let (
                response_items,
                response_ledger_info_with_sigs,
                _validator_change_events
            ) = read_client
                .update_to_latest_ledger(0, account_state_request_items).unwrap();
            for ((address, blob), response_item) in zip_eq(account_states, response_items) {
                    match response_item {
                        ResponseItem::GetAccountState {
                            account_state_with_proof,
                        } => {
                            prop_assert_eq!(&Some(blob), &account_state_with_proof.blob);
                            prop_assert!(account_state_with_proof.verify(
                                response_ledger_info_with_sigs.ledger_info(),
                                version,
                                address,
                            ).is_ok())
                        }
                        _ => unreachable!()
                    }
            }

            // Assert ledger info.
            prop_assert_eq!(ledger_info_with_sigs, &response_ledger_info_with_sigs);
         }
    }
}
