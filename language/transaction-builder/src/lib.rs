// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use ir_to_bytecode::{compiler::compile_program, parser::ast};
use lazy_static::lazy_static;
use libra_config::config::{VMConfig, VMPublishingOption};
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
        ADD_VALIDATOR_TXN_BODY, BLOCK_PROLOGUE_TXN_BODY, CREATE_ACCOUNT_TXN_BODY, MINT_TXN_BODY,
        PEER_TO_PEER_TRANSFER_TXN_BODY, PEER_TO_PEER_TRANSFER_WITH_METADATA_TXN_BODY,
        REGISTER_VALIDATOR_TXN_BODY, REMOVE_VALIDATOR_TXN_BODY, ROTATE_AUTHENTICATION_KEY_TXN_BODY,
        ROTATE_CONSENSUS_PUBKEY_TXN_BODY,
    },
};
#[cfg(any(test, feature = "fuzzing"))]
use vm::file_format::Bytecode;

lazy_static! {
    pub static ref ADD_VALIDATOR_TXN: Vec<u8> = { compile_script(&ADD_VALIDATOR_TXN_BODY) };
    static ref PEER_TO_PEER_TXN: Vec<u8> = { compile_script(&PEER_TO_PEER_TRANSFER_TXN_BODY) };
    static ref PEER_TO_PEER_WITH_METADATA_TXN: Vec<u8> =
        { compile_script(&PEER_TO_PEER_TRANSFER_WITH_METADATA_TXN_BODY) };
    static ref CREATE_ACCOUNT_TXN: Vec<u8> = { compile_script(&CREATE_ACCOUNT_TXN_BODY) };
    pub static ref REGISTER_VALIDATOR_TXN: Vec<u8> =
        { compile_script(&REGISTER_VALIDATOR_TXN_BODY) };
    pub static ref REMOVE_VALIDATOR_TXN: Vec<u8> = { compile_script(&REMOVE_VALIDATOR_TXN_BODY) };
    static ref ROTATE_AUTHENTICATION_KEY_TXN: Vec<u8> =
        { compile_script(&ROTATE_AUTHENTICATION_KEY_TXN_BODY) };
    pub static ref ROTATE_CONSENSUS_PUBKEY_TXN: Vec<u8> =
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

/// Encode a program adding `new_validator` to the pending validator set. Fails if the
/// `new_validator` address is already in the validator set, already in the pending valdiator set,
/// or does not have a `ValidatorConfig` resource stored at the address
pub fn encode_add_validator_script(new_validator: &AccountAddress) -> Script {
    Script::new(
        ADD_VALIDATOR_TXN.clone(),
        vec![TransactionArgument::Address(*new_validator)],
    )
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

/// Encode a program transferring `amount` coins from `sender` to `recipient` with associated
/// metadata `metadata`. Fails if there is no account at the recipient address or if the sender's
/// balance is lower than `amount`.
pub fn encode_transfer_with_metadata_script(
    recipient: &AccountAddress,
    amount: u64,
    metadata: Vec<u8>,
) -> Script {
    Script::new(
        PEER_TO_PEER_WITH_METADATA_TXN.clone(),
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U64(amount),
            TransactionArgument::ByteArray(ByteArray::new(metadata)),
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

/// Encode a program registering the sender as a candidate validator with the given key information.
/// `network_signing_pubkey` should be a Ed25519 public key
/// `network_identity_pubkey` should be a X25519 public key
/// `consensus_pubkey` should be a Ed25519 public c=key
pub fn encode_register_validator_script(
    consensus_pubkey: Vec<u8>,
    validator_network_signing_pubkey: Vec<u8>,
    validator_network_identity_pubkey: Vec<u8>,
    validator_network_address: Vec<u8>,
    fullnodes_network_identity_pubkey: Vec<u8>,
    fullnodes_network_address: Vec<u8>,
) -> Script {
    Script::new(
        REGISTER_VALIDATOR_TXN.clone(),
        vec![
            TransactionArgument::ByteArray(ByteArray::new(consensus_pubkey)),
            TransactionArgument::ByteArray(ByteArray::new(validator_network_signing_pubkey)),
            TransactionArgument::ByteArray(ByteArray::new(validator_network_identity_pubkey)),
            TransactionArgument::ByteArray(ByteArray::new(validator_network_address)),
            TransactionArgument::ByteArray(ByteArray::new(fullnodes_network_identity_pubkey)),
            TransactionArgument::ByteArray(ByteArray::new(fullnodes_network_address)),
        ],
    )
}

/// Encode a program adding `to_remove` to the set of pending validator removals. Fails if
/// the `to_remove` address is already in the validator set or already in the pending removals.
pub fn encode_remove_validator_script(to_remove: &AccountAddress) -> Script {
    Script::new(
        REMOVE_VALIDATOR_TXN.clone(),
        vec![TransactionArgument::Address(*to_remove)],
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
    if code == &ADD_VALIDATOR_TXN[..] {
        return "add_validator_transaction".to_string();
    } else if code == &PEER_TO_PEER_TXN[..] {
        return "peer_to_peer_transaction".to_string();
    } else if code == &PEER_TO_PEER_WITH_METADATA_TXN[..] {
        return "peer_to_peer_with_metadata_transaction".to_string();
    } else if code == &CREATE_ACCOUNT_TXN[..] {
        return "create_account_transaction".to_string();
    } else if code == &MINT_TXN[..] {
        return "mint_transaction".to_string();
    } else if code == &REMOVE_VALIDATOR_TXN[..] {
        return "remove_validator_transaction".to_string();
    } else if code == &ROTATE_AUTHENTICATION_KEY_TXN[..] {
        return "rotate_authentication_key_transaction".to_string();
    } else if code == &ROTATE_CONSENSUS_PUBKEY_TXN[..] {
        return "rotate_consensus_pubkey_transaction".to_string();
    }
    "<unknown transaction>".to_string()
}

pub fn allowing_script_hashes() -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
    vec![
        ADD_VALIDATOR_TXN.clone(),
        MINT_TXN.clone(),
        PEER_TO_PEER_TXN.clone(),
        PEER_TO_PEER_WITH_METADATA_TXN.clone(),
        REMOVE_VALIDATOR_TXN.clone(),
        REGISTER_VALIDATOR_TXN.clone(),
        ROTATE_AUTHENTICATION_KEY_TXN.clone(),
        ROTATE_CONSENSUS_PUBKEY_TXN.clone(),
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
    }
}
