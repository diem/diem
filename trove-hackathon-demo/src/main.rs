// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(unused_imports)]

use anyhow::{Context, Result};
use diem_config::config::NodeConfig;
use diem_crypto::PrivateKey;
use diem_sdk::move_types::identifier::Identifier;
use diem_sdk::move_types::language_storage::StructTag;
use diem_sdk::types::account_config::{xus_tag, CORE_CODE_ADDRESS};
use diem_sdk::{
    client::BlockingClient,
    transaction_builder::{Currency, TransactionFactory},
    types::LocalAccount,
};
use diem_transaction_builder::stdlib;
use diem_types::transaction::Module;
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
use std::fs;
use std::path::{Path, PathBuf};
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
    InitMultiToken {},

    PublishModule {
        #[structopt(long)]
        path: PathBuf,
    },

    CreateAccount {
        new_account_address: String,
        new_auth_key_prefix: String,
    },

    CreateBasicAccount {
        new_account_address: String,
        new_auth_key_prefix: String,
    },

    RegisterBarsUser {},

    MintBarsNft {
        #[structopt(long)]
        creator_addr: String,
        #[structopt(long)]
        creator_name: String,
        #[structopt(long)]
        content_uri: String,
        #[structopt(long)]
        amount: u64,
    },

    /// Transfer a BARS NFT
    TransferBarsNft {
        #[structopt(long)]
        to: String,
        #[structopt(long)]
        amount: u64,
        #[structopt(long)]
        creator: String,
        #[structopt(long)]
        creation_num: u64,
    },

    ModulePublishSetApproval {
        #[structopt(long)]
        enable: bool,
    },

    ProposeApproveModule {
        #[structopt(long)]
        path: PathBuf,
    },

    VoteApproveModule {
        #[structopt(long)]
        path: PathBuf,
        #[structopt(long)]
        counter: u64,
    },

    ApproveModule {
        #[structopt(long)]
        path: PathBuf,
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
        Command::RegisterBarsUser { .. } => register_bars_user(&mut account, &client)?,
        Command::MintBarsNft { creator_addr, creator_name, content_uri, amount } => mint_bars_nft(&mut account, &client, creator_addr, creator_name, content_uri, amount)?,
        Command::TransferBarsNft {
            to,
            amount,
            creator,
            creation_num,
        } => transfer_bars_nft(&mut account, &client, to, amount, creator, creation_num)?,
        Command::CreateAccount {
            new_account_address,
            new_auth_key_prefix,
        } => create_account(
            &mut account,
            &client,
            new_account_address,
            new_auth_key_prefix,
        )?,
        Command::CreateBasicAccount {
            new_account_address,
            new_auth_key_prefix,
        } => create_basic_account(
            &mut account,
            &client,
            new_account_address,
            new_auth_key_prefix,
        )?,
        Command::PublishModule { path } => publish_module(&mut account, &client, path)?,
        Command::ModulePublishSetApproval { enable } => {
            module_publish_set_approval(&mut account, &client, enable)?
        }
        Command::ApproveModule { path } => approve_module(&mut account, &client, path)?,
        Command::ProposeApproveModule { path } => {
            propose_approve_module(&mut account, &client, path)?
        }
        Command::VoteApproveModule { path, counter } => {
            vote_approve_module(&mut account, &client, path, counter)?
        }
    }

    Ok(())
}

fn propose_approve_module(
    account: &mut LocalAccount,
    client: &BlockingClient,
    path: PathBuf,
) -> Result<()> {
    let txn =
        account.sign_with_transaction_builder(TransactionFactory::new(ChainId::test()).payload(
            stdlib::encode_propose_pre_approve_module_publish_script_function(
                calc_module_hash(path),
            ),
        ));
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn vote_approve_module(
    account: &mut LocalAccount,
    client: &BlockingClient,
    path: PathBuf,
    counter: u64,
) -> Result<()> {
    let txn =
        account.sign_with_transaction_builder(TransactionFactory::new(ChainId::test()).payload(
            stdlib::encode_vote_pre_approve_module_publish_script_function(
                calc_module_hash(path),
                counter,
            ),
        ));
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn approve_module(account: &mut LocalAccount, client: &BlockingClient, path: PathBuf) -> Result<()> {
    let txn =
        account.sign_with_transaction_builder(TransactionFactory::new(ChainId::test()).payload(
            stdlib::encode_pre_approve_module_publish_script_function(calc_module_hash(path)),
        ));
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn module_publish_set_approval(
    account: &mut LocalAccount,
    client: &BlockingClient,
    enable: bool,
) -> Result<()> {
    let txn = account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test())
            .payload(stdlib::encode_set_module_publish_pre_approval_script_function(enable)),
    );
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn transfer_bars_nft(
    account: &mut LocalAccount,
    client: &BlockingClient,
    to: String,
    amount: u64,
    creator: String,
    creation_num: u64,
) -> Result<()> {
    let token = TypeTag::Struct(StructTag {
        address: AccountAddress::from_hex_literal("0x1").unwrap(),
        module: Identifier::new("BARSToken").unwrap(),
        name: Identifier::new("BARSToken").unwrap(),
        type_params: Vec::new(),
    });
    let txn =
        account.sign_with_transaction_builder(TransactionFactory::new(ChainId::test()).payload(
            stdlib::encode_transfer_token_between_galleries_script_function(
                token,
                AccountAddress::from_hex_literal(&to).unwrap(),
                amount,
                AccountAddress::from_hex_literal(&creator).unwrap(),
                creation_num,
            ),
        ));
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn publish_module(
    account: &mut LocalAccount,
    client: &BlockingClient,
    path: PathBuf,
) -> Result<()> {
    let input_file_path: &Path = path.as_ref();
    let data = fs::read(input_file_path).expect("Unable to read module file");

    let txn = account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test())
            .payload(TransactionPayload::Module(Module::new(data))),
    );
    send(&client, txn)?;
    println!("Success");
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

fn create_basic_account(
    account: &mut LocalAccount,
    client: &BlockingClient,
    new_address: String,
    new_auth_key_prefix: String,
) -> Result<()> {
    let txn =
        account.sign_with_transaction_builder(TransactionFactory::new(ChainId::test()).payload(
            stdlib::encode_create_account_script_function(
                xus_tag(),
                AccountAddress::from_hex_literal(&new_address).unwrap(),
                hex::decode(&new_auth_key_prefix).unwrap(),
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

fn register_bars_user(account: &mut LocalAccount, client: &BlockingClient) -> Result<()> {
    let txn = account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test())
            .payload(stdlib::encode_register_bars_user_script_function()),
    );
    send(&client, txn)?;
    println!("Success");
    Ok(())
}

fn mint_bars_nft(account: &mut LocalAccount, client: &BlockingClient, creator_addr: String, creator_name: String, content_uri: String, amount: u64) -> Result<()> {
    let txn = account.sign_with_transaction_builder(
        TransactionFactory::new(ChainId::test()).payload(stdlib::encode_mint_bars_script_function(
            AccountAddress::from_hex_literal(&creator_addr).unwrap(),
            creator_name.as_bytes().to_vec(),
            content_uri.as_bytes().to_vec(),
            amount,
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

/// Sha3-256 module hash, the same way VM executor is doing it
fn calc_module_hash(path: PathBuf) -> Vec<u8> {
    let data = fs::read(path).expect("Unable to read module file");
    let module = Module::new(data);
    diem_crypto::HashValue::sha3_256_of(module.code()).to_vec()
}