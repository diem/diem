// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use diem_config::config::NodeConfig;
use diem_crypto::PrivateKey;
use diem_sdk::{
    client::BlockingClient,
    transaction_builder::{Currency, TransactionFactory},
    types::LocalAccount,
};
use diem_types::{
    account_config, chain_id::ChainId, transaction::authenticator::AuthenticationKey,
};
use generate_key::load_key;
use shuffle_transaction_builder::framework::{
    encode_create_regular_account_script_function, encode_mint_coin_script_function,
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Fancy coin app")]
pub struct SampleApp {
    /// Directory where the node config lives.
    #[structopt(long = "node-config-dir")]
    node_config_dir: PathBuf,
    #[structopt(long = "account-key-path")]
    account_key_path: PathBuf,
}

fn main() -> Result<()> {
    let args = SampleApp::from_args();
    let config_path = args.node_config_dir.join("0").join("node.yaml");
    let config = NodeConfig::load(&config_path)
        .with_context(|| format!("Failed to load NodeConfig from file: {:?}", config_path))?;
    let root_key_path = args.node_config_dir.join("mint.key");
    let root_account_key = load_key(root_key_path);
    let new_account_key = load_key(args.account_key_path);
    let json_rpc_url = format!("http://0.0.0.0:{}", config.json_rpc.address.port());
    println!("Connecting to {}...", json_rpc_url);

    let client = BlockingClient::new(json_rpc_url);
    let root_seq_num = client
        .get_account(account_config::diem_root_address())?
        .into_inner()
        .unwrap()
        .sequence_number;
    let mut root_account = LocalAccount::new(
        account_config::diem_root_address(),
        root_account_key,
        root_seq_num,
    );
    let mut new_account = LocalAccount::new(
        AuthenticationKey::ed25519(&new_account_key.public_key()).derived_address(),
        new_account_key,
        0,
    );

    // Create a new account.
    print!("Create a new account...");
    let create_new_account_txn = root_account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test()).payload(
            encode_create_regular_account_script_function(
                Currency::XUS.type_tag(),
                new_account.address(),
                new_account.authentication_key().prefix().to_vec(),
            ),
        ),
    );
    send(&client, create_new_account_txn)?;
    println!("Success!");

    // Mint a coin to the newly created account.
    print!("Mint a coin to the new account...");
    let mint_coin_txn = new_account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test()).payload(encode_mint_coin_script_function(100)),
    );
    send(&client, mint_coin_txn)?;
    println!("Success!");

    // This fails because a coin resource has already been published to the new account
    // Mint a coin to the newly created account.
    println!("Mint another coin to the new account (this should fail)...");
    let mint_another_coin_txn = new_account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test()).payload(encode_mint_coin_script_function(42)),
    );
    send(&client, mint_another_coin_txn)?;
    // Should not reach here
    println!("Success!");
    Ok(())
}

/// Send a transaction to the blockchain through the blocking client.
fn send(client: &BlockingClient, tx: diem_types::transaction::SignedTransaction) -> Result<()> {
    use diem_json_rpc_types::views::VMStatusView;

    client.submit(&tx)?;
    assert_eq!(
        client
            .wait_for_signed_transaction(&tx, Some(std::time::Duration::from_secs(60)), None)?
            .into_inner()
            .vm_status,
        VMStatusView::Executed,
    );
    Ok(())
}
