// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use ir_to_bytecode::{compiler::compile_program, parser::ast};
use lazy_static::lazy_static;
use libra_config::config::{VMConfig, VMMode, VMPublishingOption};
use libra_crypto::HashValue;
use libra_types::block_metadata::BlockMetadata;
use libra_types::{
    account_address::AccountAddress,
    byte_array::ByteArray,
    transaction::{Script, Transaction, TransactionArgument, SCRIPT_HASH_LENGTH},
};
use std::{collections::HashSet, iter::FromIterator};
use stdlib::{
    stdlib_modules,
    transaction_scripts::{
        BLOCK_PROLOGUE_TXN_BODY, CREATE_ACCOUNT_TXN_BODY, MINT_TXN_BODY,
        PEER_TO_PEER_TRANSFER_TXN_BODY, ROTATE_AUTHENTICATION_KEY_TXN_BODY,
        ROTATE_CONSENSUS_PUBKEY_TXN_BODY,
    },
};
#[cfg(any(test, feature = "fuzzing"))]
use vm::file_format::Bytecode;

lazy_static! {
    static ref PEER_TO_PEER_TXN: Vec<u8> = { compile_script(&PEER_TO_PEER_TRANSFER_TXN_BODY) };
    static ref CREATE_ACCOUNT_TXN: Vec<u8> = { compile_script(&CREATE_ACCOUNT_TXN_BODY) };
    static ref ROTATE_AUTHENTICATION_KEY_TXN: Vec<u8> =
        { compile_script(&ROTATE_AUTHENTICATION_KEY_TXN_BODY) };
    static ref ROTATE_CONSENSUS_PUBKEY_TXN: Vec<u8> =
        { compile_script(&ROTATE_CONSENSUS_PUBKEY_TXN_BODY) };
    static ref MINT_TXN: Vec<u8> = { compile_script(&MINT_TXN_BODY) };
    static ref BLOCK_PROLOGUE_TXN: Vec<u8> = { compile_script(&BLOCK_PROLOGUE_TXN_BODY) };
}

fn compile_script(body: &ast::Program) -> Vec<u8> {
    let compiled_program =
        compile_program(AccountAddress::default(), body.clone(), stdlib_modules())
            .unwrap()
            .0;
    let mut script_bytes = vec![];
    compiled_program
        .script
        .serialize(&mut script_bytes)
        .unwrap();
    script_bytes
}

/// Encode a program transferring `amount` coins from `sender` to `recipient`. Fails if there is no
/// account at the recipient address or if the sender's balance is lower than `amount`.
pub fn encode_transfer_script(recipient: &AccountAddress, amount: u64) -> Script {
    Script::new(
        PEER_TO_PEER_TXN.clone(),
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U64(amount),
        ],
    )
}

/// Encode a program transferring `amount` coins from `sender` to `recipient` but padd the output
/// bytecode with unreachable instructions.
#[cfg(any(test, feature = "fuzzing"))]
pub fn encode_transfer_script_with_padding(
    recipient: &AccountAddress,
    amount: u64,
    padding_size: u64,
) -> Script {
    let mut script_mut = compile_program(
        AccountAddress::default(),
        PEER_TO_PEER_TRANSFER_TXN_BODY.clone(),
        stdlib_modules(),
    )
    .unwrap()
    .0
    .script
    .into_inner();
    script_mut
        .main
        .code
        .code
        .extend(std::iter::repeat(Bytecode::Ret).take(padding_size as usize));
    let mut script_bytes = vec![];
    script_mut
        .freeze()
        .unwrap()
        .serialize(&mut script_bytes)
        .unwrap();

    Script::new(
        script_bytes,
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U64(amount),
        ],
    )
}

/// Encode a program creating a fresh account at `account_address` with `initial_balance` coins
/// transferred from the sender's account balance. Fails if there is already an account at
/// `account_address` or if the sender's balance is lower than `initial_balance`.
pub fn encode_create_account_script(
    account_address: &AccountAddress,
    initial_balance: u64,
) -> Script {
    Script::new(
        CREATE_ACCOUNT_TXN.clone(),
        vec![
            TransactionArgument::Address(*account_address),
            TransactionArgument::U64(initial_balance),
        ],
    )
}

/// Encode a program that rotates the sender's consensus public key to `new_key`.
pub fn encode_rotate_consensus_pubkey_script(new_key: Vec<u8>) -> Script {
    Script::new(
        ROTATE_CONSENSUS_PUBKEY_TXN.clone(),
        vec![TransactionArgument::ByteArray(ByteArray::new(new_key))],
    )
}

/// Encode a program that rotates the sender's authentication key to `new_key`. `new_key` should be
/// a 256 bit sha3 hash of an ed25519 public key.
pub fn rotate_authentication_key_script(new_hashed_key: Vec<u8>) -> Script {
    Script::new(
        ROTATE_AUTHENTICATION_KEY_TXN.clone(),
        vec![TransactionArgument::ByteArray(ByteArray::new(
            new_hashed_key,
        ))],
    )
}

// TODO: this should go away once we are no longer using it in tests
/// Encode a program creating `amount` coins for sender
pub fn encode_mint_script(sender: &AccountAddress, amount: u64) -> Script {
    Script::new(
        MINT_TXN.clone(),
        vec![
            TransactionArgument::Address(*sender),
            TransactionArgument::U64(amount),
        ],
    )
}

// TODO: this should go away once we are no longer using it in tests
pub fn encode_block_prologue_script(block_metadata: BlockMetadata) -> Transaction {
    Transaction::BlockMetadata(block_metadata)
}

/// Returns a user friendly mnemonic for the transaction type if the transaction is
/// for a known, white listed, transaction.
pub fn get_transaction_name(code: &[u8]) -> String {
    if code == &PEER_TO_PEER_TXN[..] {
        return "peer_to_peer_transaction".to_string();
    } else if code == &CREATE_ACCOUNT_TXN[..] {
        return "create_account_transaction".to_string();
    } else if code == &MINT_TXN[..] {
        return "mint_transaction".to_string();
    } else if code == &ROTATE_AUTHENTICATION_KEY_TXN[..] {
        return "rotate_authentication_key_transaction".to_string();
    }
    "<unknown transaction>".to_string()
}

pub fn allowing_script_hashes() -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
    vec![
        MINT_TXN.clone(),
        PEER_TO_PEER_TXN.clone(),
        ROTATE_AUTHENTICATION_KEY_TXN.clone(),
        CREATE_ACCOUNT_TXN.clone(),
    ]
    .into_iter()
    .map(|s| *HashValue::from_sha3_256(&s).as_ref())
    .collect()
}

pub fn default_config() -> VMConfig {
    VMConfig {
        publishing_options: VMPublishingOption::Locked(HashSet::from_iter(
            allowing_script_hashes().into_iter(),
        )),
        mode: VMMode::Onchain,
    }
}
