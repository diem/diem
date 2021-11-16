// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(unused_imports)]

use anyhow::Result;
use diem_sdk::{
    client::BlockingClient,
    transaction_builder::TransactionFactory,
    types::{account_config::xus_tag, LocalAccount},
};
use diem_types::{
    account_address::AccountAddress,
    chain_id::ChainId,
    transaction::{ScriptFunction, TransactionPayload},
};
use generate_key::load_key;
use move_core_types::{
    ident_str,
    language_storage::{ModuleId, TypeTag},
};
use std::path::PathBuf;
use structopt::StructOpt;

pub mod request_utils;

#[derive(Debug, StructOpt)]
#[structopt(about = "Demo for trove hackathon")]
pub struct DemoCli {
    #[structopt(long)]
    account_key_path: PathBuf,
    #[structopt(long)]
    account_address: String,
    #[structopt(long, default_value = "http://0.0.0.0:8080")]
    jsonrpc_endpoint: String,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    CreateBasicAccount {
        new_account_address: String,
        new_auth_key_prefix: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: DemoCli = DemoCli::from_args();
    let account_key = load_key(args.account_key_path);
    let address = AccountAddress::from_hex_literal(&args.account_address).unwrap();

    let json_rpc_url = args.jsonrpc_endpoint;
    println!("Connecting to {}...", json_rpc_url);

    let client = BlockingClient::new(json_rpc_url.clone());

    let seq_num =
        request_utils::get_account_seq_num(json_rpc_url.as_str(), args.account_address).await?;
    let mut account = LocalAccount::new(address, account_key, seq_num);

    match args.cmd {
        Command::CreateBasicAccount {
            new_account_address,
            new_auth_key_prefix,
        } => create_basic_account(
            &mut account,
            &client,
            new_account_address,
            new_auth_key_prefix,
        )?,
    }

    Ok(())
}

fn create_basic_account(
    account: &mut LocalAccount,
    client: &BlockingClient,
    new_address: String,
    new_auth_key_prefix: String,
) -> Result<()> {
    let txn = account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test()).payload(encode_create_account_script_function(
            xus_tag(),
            AccountAddress::from_hex_literal(&new_address).unwrap(),
            hex::decode(&new_auth_key_prefix).unwrap(),
        )),
    );
    send(client, txn)?;
    println!("Success");
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

pub fn encode_create_account_script_function(
    coin_type: TypeTag,
    new_account_address: AccountAddress,
    auth_key_prefix: Vec<u8>,
) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        ModuleId::new(
            AccountAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
            ident_str!("AccountCreationScripts").to_owned(),
        ),
        ident_str!("create_account").to_owned(),
        vec![coin_type],
        vec![
            bcs::to_bytes(&new_account_address).unwrap(),
            bcs::to_bytes(&auth_key_prefix).unwrap(),
        ],
    ))
}
