// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod integration_test_impl;

use executor::db_bootstrapper::{generate_waypoint, maybe_bootstrap};
use executor_types::StateComputeResult;
use libra_config::{config::NodeConfig, utils};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    HashValue,
};
use libra_types::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    block_metadata::BlockMetadata,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{Script, Transaction},
    validator_signer::ValidatorSigner,
    waypoint::Waypoint,
};
use libra_vm::{LibraVM, VMExecutor};
use libradb::LibraDB;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    thread::JoinHandle,
};
use storage_interface::{DbReader, DbReaderWriter};
use storage_service::start_storage_service_with_db;

/// Helper function for test to blindly bootstrap without waypoint.
pub fn bootstrap_genesis<V: VMExecutor>(
    db: &DbReaderWriter,
    genesis_txn: &Transaction,
) -> anyhow::Result<Waypoint> {
    let waypoint = generate_waypoint::<V>(db, genesis_txn)?;
    maybe_bootstrap::<V>(db, genesis_txn, waypoint)?;
    Ok(waypoint)
}

pub fn start_storage_service() -> (NodeConfig, JoinHandle<()>, Arc<dyn DbReader>) {
    let (mut config, _genesis_key) = config_builder::test_config();
    let tmp_dir = libra_temppath::TempPath::new();
    config.storage.dir = tmp_dir.path().to_path_buf();

    let server_port = utils::get_available_port();
    config.storage.address = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), server_port);
    let (db, db_rw) = DbReaderWriter::wrap(LibraDB::new_for_test(&config.storage.dir()));
    bootstrap_genesis::<LibraVM>(&db_rw, utils::get_genesis_txn(&config).unwrap()).unwrap();
    let handle = start_storage_service_with_db(&config, db.clone());
    (config, handle, db as Arc<dyn DbReader>)
}

pub fn gen_block_id(index: u8) -> HashValue {
    HashValue::new([index; HashValue::LENGTH])
}

pub fn gen_ledger_info_with_sigs(
    epoch: u64,
    output: StateComputeResult,
    commit_block_id: HashValue,
    signer: Vec<&ValidatorSigner>,
) -> LedgerInfoWithSignatures {
    let ledger_info = LedgerInfo::new(
        BlockInfo::new(
            epoch,
            0, /* round */
            commit_block_id,
            output.root_hash(),
            output.version(),
            0, /* timestamp */
            output.epoch_state().clone(),
        ),
        HashValue::zero(),
    );
    let signatures = signer
        .iter()
        .map(|s| (s.author(), s.sign(&ledger_info)))
        .collect();
    LedgerInfoWithSignatures::new(ledger_info, signatures)
}

pub fn extract_signer(config: &mut NodeConfig) -> ValidatorSigner {
    let sr_test = config.consensus.safety_rules.test.as_ref().unwrap();
    ValidatorSigner::new(
        sr_test.author,
        sr_test.consensus_key.as_ref().unwrap().private_key(),
    )
}

pub fn gen_block_metadata(index: u8, proposer: AccountAddress) -> BlockMetadata {
    BlockMetadata::new(
        gen_block_id(index),
        index as u64,
        index as u64,
        vec![],
        proposer,
    )
}

pub fn get_test_signed_transaction(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    program: Option<Script>,
) -> Transaction {
    Transaction::UserTransaction(get_test_signed_txn(
        sender,
        sequence_number,
        &private_key,
        public_key,
        program,
    ))
}
