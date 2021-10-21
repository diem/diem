// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(unused_imports)]
use anyhow::{Context, Result};
use diem_config::config::NodeConfig;
use diem_crypto::PrivateKey;
use diem_sdk::types::account_config::xus_tag;
use diem_sdk::{
    client::BlockingClient,
    transaction_builder::{Currency, TransactionFactory},
    types::LocalAccount,
};
use diem_transaction_builder::stdlib;
use diem_types::{
    account_address::AccountAddress,
    transaction::{Script, ScriptFunction, TransactionArgument, TransactionPayload, VecBytes},
};
use diem_types::{
    account_config, chain_id::ChainId, transaction::authenticator::AuthenticationKey,
};
use generate_key::load_key;
use move_core_types::{
    ident_str,
    language_storage::{ModuleId, TypeTag},
};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Demo for trove hackathon")]
pub struct TroveHackathonDemo {
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
    /// Create an account on the blockchain
    InitMultiToken {},

    RegisterUser {},

    MintBarsNft {},

    CreateAccount {
        new_account_address: String,
        new_auth_key_prefix: String,
    },

    /// Transfer a BARS NFT
    TransferBarsNft {
        #[structopt(long)]
        address_from: String,
        #[structopt(long)]
        address_to: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: TroveHackathonDemo = TroveHackathonDemo::from_args();
    let account_key = load_key(args.account_key_path);
    let address = AccountAddress::from_hex_literal(&args.account_address).unwrap();

    let json_rpc_url = args.jsonrpc_endpoint;
    println!("Connecting to {}...", json_rpc_url);

    let client = BlockingClient::new(json_rpc_url);

    let seq_num = client
        .get_account(address)?
        .into_inner()
        .unwrap()
        .sequence_number;
    let mut account = LocalAccount::new(address, account_key, seq_num);

    match args.cmd {
        Command::InitMultiToken { .. } => init_multi_token(&mut account, &client)?,
        Command::RegisterUser { .. } => register_user(&mut account, &client)?,
        Command::MintBarsNft { .. } => mint_bars_nft(&mut account, &client)?,
        Command::TransferBarsNft { .. } => {
            // transfer_bars_nft()
        }
        Command::CreateAccount {
            new_account_address,
            new_auth_key_prefix,
        } => create_account(&mut account, &client, new_account_address, new_auth_key_prefix)?,
    }

    Ok(())
}

fn create_account(
    account: &mut LocalAccount,
    client: &BlockingClient,
    new_address: String,
    new_auth_key_prefix: String,
) -> Result<()> {
    let txn =
        account.sign_with_transaction_builder(TransactionFactory::new(ChainId::test()).payload(
            stdlib::encode_create_parent_vasp_account_script_function(
                xus_tag(),
                0,
                AccountAddress::from_hex_literal(&new_address).unwrap(),
                hex::decode(&new_auth_key_prefix).unwrap(),
                Vec::new(),
                true,
            ),
        ));
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn init_multi_token(account: &mut LocalAccount, client: &BlockingClient) -> Result<()> {
    let txn = account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test())
            .payload(stdlib::encode_nft_initialize_script_function()),
    );
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn register_user(account: &mut LocalAccount, client: &BlockingClient) -> Result<()> {
    let txn = account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test())
            .payload(stdlib::encode_register_user_script_function()),
    );
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn mint_bars_nft(account: &mut LocalAccount, client: &BlockingClient) -> Result<()> {
    let txn = account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test()).payload(stdlib::encode_mint_bars_script_function(
            "Ankush".to_string().as_bytes().to_vec(),
            "diem.com".to_string().as_bytes().to_vec(),
            100,
        )),
    );
    send(&client, txn)?;
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
