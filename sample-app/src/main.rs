// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use diem_config::config::NodeConfig;
use diem_sdk::{
    client::BlockingClient,
    move_types::{ident_str, language_storage::ModuleId},
    transaction_builder::TransactionFactory,
    types::LocalAccount,
};
use diem_types::{
    account_address::AccountAddress,
    account_config,
    chain_id::ChainId,
    transaction::{ScriptFunction, TransactionPayload},
};
use generate_key::load_key;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Fancy coin app")]
pub struct SampleApp {
    /// Directory where the node config lives.
    #[structopt(long = "node-config-dir")]
    node_config_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = SampleApp::from_args();
    let config_path = args.node_config_dir.join("0").join("node.yaml");
    let config = NodeConfig::load(&config_path)
        .with_context(|| format!("Failed to load NodeConfig from file: {:?}", config_path))?;
    let root_key_path = args.node_config_dir.join("mint.key");
    let root_account_key = load_key(root_key_path);
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

    // Create a new account.

    let create_new_account_txn = root_account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test()).payload(TransactionPayload::ScriptFunction(
            ScriptFunction::new(
                ModuleId::new(
                    AccountAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
                    ident_str!("NFT").to_owned(),
                ),
                ident_str!("publish_dmitri_nft").to_owned(),
                vec![],
                vec![],
            ),
        )),
    );
    send(&client, create_new_account_txn)?;
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
