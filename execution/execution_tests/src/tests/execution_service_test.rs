// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{create_and_start_server, gen_block_id, gen_ledger_info_with_sigs};
use config_builder::util::get_test_config;
use crypto::{ed25519::*, hash::GENESIS_BLOCK_ID, test_utils::KeyPair};
use execution_client::ExecutionClient;
use execution_proto::ExecuteBlockRequest;
use futures01::future::Future;
use grpcio::EnvBuilder;
use std::sync::Arc;
use types::{
    account_address::AccountAddress,
    account_config,
    transaction::{RawTransaction, SignedTransaction},
};
use vm_genesis::encode_mint_program;

fn encode_mint_transaction(
    seqnum: u64,
    sender_keypair: &KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
) -> SignedTransaction {
    let (_privkey, pubkey) = compat::generate_keypair(None);
    let sender = account_config::association_address();
    let receiver = AccountAddress::from_public_key(&pubkey);
    let program = encode_mint_program(&receiver, 100);
    let raw_txn = RawTransaction::new(
        sender,
        seqnum,
        program,
        /* max_gas_amount = */ 100_000,
        /* gas_unit_price = */ 1,
        std::time::Duration::from_secs(u64::max_value()),
    );
    raw_txn
        .sign(
            &sender_keypair.private_key,
            sender_keypair.public_key.clone(),
        )
        .expect("Signing should work.")
        .into_inner()
}

#[test]
fn test_execution_service_basic() {
    let (config, faucet_keypair) = get_test_config();
    let (_storage_server_handle, mut execution_server) = create_and_start_server(&config);

    let execution_client = ExecutionClient::new(
        Arc::new(EnvBuilder::new().build()),
        &config.execution.address,
        config.execution.port,
    );

    let parent_block_id = *GENESIS_BLOCK_ID;
    let block_id = gen_block_id(1);
    let version = 100;

    let txns = (0..version)
        .map(|i| encode_mint_transaction(i + 1, &faucet_keypair))
        .collect();
    let execute_block_request = ExecuteBlockRequest::new(txns, parent_block_id, block_id);
    let execute_block_response = execution_client
        .execute_block(execute_block_request)
        .unwrap();

    let ledger_info_with_sigs = gen_ledger_info_with_sigs(
        u64::from(version),
        execute_block_response.root_hash(),
        block_id,
    );
    execution_client
        .commit_block(ledger_info_with_sigs)
        .unwrap();

    execution_server.shutdown().wait().unwrap();
}
