// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use itertools::zip_eq;
use libra_config::{config::NodeConfig, utils};
use libra_crypto::hash::CryptoHash;
#[cfg(test)]
use libradb::test_helper::arb_blocks_to_commit;
use proptest::prelude::*;
use simple_storage_client::SimpleStorageClient;
use std::{
    collections::{BTreeMap, HashMap},
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

fn start_test_storage_with_client() -> (
    JoinHandle<()>,
    libra_temppath::TempPath,
    SimpleStorageClient,
) {
    let mut config = NodeConfig::random();
    let tmp_dir = libra_temppath::TempPath::new();

    let server_port = utils::get_available_port();
    config.storage.simple_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);

    let db = Arc::new(LibraDB::new_for_test(&tmp_dir));
    let storage_server_handle = start_simple_storage_service_with_db(&config, db);

    let client = SimpleStorageClient::new(&config.storage.simple_address);
    (storage_server_handle, tmp_dir, client)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]
    #[test]
    fn test_simple_storage_service(blocks in arb_blocks_to_commit().no_shrink()) {
        let (_handle, _tmp_dir, client) =
            start_test_storage_with_client();

        let mut version = 0;
        let mut all_accounts = BTreeMap::new();
        let mut all_txns = vec![];

        for (txns_to_commit, ledger_info_with_sigs) in &blocks {
            client.save_transactions(
                txns_to_commit.clone(),
                version, /* first_version */
                Some(ledger_info_with_sigs.clone()),
            ).unwrap();
            version += txns_to_commit.len() as u64;
            let mut account_states = HashMap::new();
            // Get the ground truth of account states.
            txns_to_commit.iter().for_each(|txn_to_commit| {
                account_states.extend(txn_to_commit.account_states().clone())
            });

            // Record all account states.
            for (address, blob) in account_states.iter() {
                all_accounts.insert(address.hash(), blob.clone());
            }

            // Record all transactions.
            all_txns.extend(
                txns_to_commit
                    .iter()
                    .map(|txn_to_commit| txn_to_commit.transaction().clone()),
            );

            let account_states_returned = account_states
                .keys()
                .map(|address| client.get_account_state_with_proof_by_version(*address, version - 1).unwrap())
                .collect::<Vec<_>>();
            let startup_info = client.get_startup_info().unwrap().unwrap();
            for ((address, blob), state_with_proof) in zip_eq(account_states, account_states_returned) {
                 prop_assert_eq!(&Some(blob), &state_with_proof.0);
                 prop_assert!(state_with_proof.1
                     .verify(
                         startup_info.committed_tree_state.account_state_root_hash,
                         address.hash(),
                         state_with_proof.0.as_ref()
                     )
                     .is_ok());
            }
        }
    }
}
