// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::{account::Account, gas_costs};
use compiled_stdlib::transaction_scripts::StdlibScript;
use compiler::Compiler;
use libra_types::{
    account_address::AccountAddress,
    account_config,
    account_config::{lbr_type_tag, LBR_NAME},
    transaction::{RawTransaction, SignedTransaction, TransactionArgument},
};
use once_cell::sync::Lazy;

pub static CREATE_ACCOUNT_SCRIPT: Lazy<Vec<u8>> = Lazy::new(|| {
    let code = "
    import 0x1.Libra;
    import 0x1.LibraAccount;

    main<Token>(account: &signer, fresh_address: address, auth_key_prefix: vector<u8>, initial_amount: u64) {
      let with_cap: LibraAccount.WithdrawCapability;
      LibraAccount.create_unhosted_account<Token>(copy(account), copy(fresh_address), move(auth_key_prefix), false);
      if (copy(initial_amount) > 0) {
         with_cap = LibraAccount.extract_withdraw_capability(copy(account));
         LibraAccount.deposit<Token>(
           copy(account),
           move(fresh_address),
           LibraAccount.withdraw_from<Token>(&with_cap, move(initial_amount))
         );
         LibraAccount.restore_withdraw_capability(move(with_cap));
      }
      return;
    }
";

    let compiler = Compiler {
        address: account_config::CORE_CODE_ADDRESS,
        extra_deps: vec![],
        ..Compiler::default()
    };
    compiler
        .into_script_blob("file_name", code)
        .expect("Failed to compile")
});

pub static EMPTY_SCRIPT: Lazy<Vec<u8>> = Lazy::new(|| {
    let code = "
    main<Token>(account: &signer) {
      return;
    }
";

    let compiler = Compiler {
        address: account_config::CORE_CODE_ADDRESS,
        extra_deps: vec![],
        ..Compiler::default()
    };
    compiler
        .into_script_blob("file_name", code)
        .expect("Failed to compile")
});

/// Returns a transaction to add a new validator
pub fn add_validator_txn(
    sender: &Account,
    new_validator: &Account,
    seq_num: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_validator.address()));

    sender.create_signed_txn_with_args(
        StdlibScript::AddValidator.compiled_bytes().into_vec(),
        vec![],
        args,
        seq_num,
        gas_costs::TXN_RESERVED * 2,
        0,
        LBR_NAME.to_owned(),
    )
}

/// Returns a transaction to update validators' configs and reconfigure
///   (= emit reconfiguration event and change the epoch)
pub fn reconfigure_txn(sender: &Account, seq_num: u64) -> SignedTransaction {
    sender.create_signed_txn_with_args(
        StdlibScript::Reconfigure.compiled_bytes().into_vec(),
        vec![],
        Vec::new(),
        seq_num,
        gas_costs::TXN_RESERVED * 2,
        0,
        LBR_NAME.to_owned(),
    )
}

pub fn empty_txn(
    sender: &Account,
    seq_num: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_currency_code: String,
) -> SignedTransaction {
    sender.create_signed_txn_with_args(
        EMPTY_SCRIPT.to_vec(),
        vec![],
        vec![],
        seq_num,
        max_gas_amount,
        gas_unit_price,
        gas_currency_code,
    )
}

/// Returns a transaction to create a new account with the given arguments.
pub fn create_account_txn(
    sender: &Account,
    new_account: &Account,
    seq_num: u64,
    initial_amount: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_account.address()));
    args.push(TransactionArgument::U8Vector(new_account.auth_key_prefix()));
    args.push(TransactionArgument::U64(initial_amount));

    sender.create_signed_txn_with_args(
        CREATE_ACCOUNT_SCRIPT.to_vec(),
        vec![lbr_type_tag()],
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        0,
        LBR_NAME.to_owned(),
    )
}

/// Returns a transaction to create a validator account with the given arguments.
pub fn create_validator_account_txn(
    sender: &Account,
    new_account: &Account,
    seq_num: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_account.address()));
    args.push(TransactionArgument::U8Vector(new_account.auth_key_prefix()));

    sender.create_signed_txn_with_args(
        StdlibScript::CreateValidatorAccount
            .compiled_bytes()
            .into_vec(),
        vec![lbr_type_tag()],
        args,
        seq_num,
        gas_costs::TXN_RESERVED * 3,
        0,
        LBR_NAME.to_owned(),
    )
}

/// Returns a transaction to transfer coin from one account to another (possibly new) one, with the
/// given arguments.
pub fn peer_to_peer_txn(
    sender: &Account,
    receiver: &Account,
    seq_num: u64,
    transfer_amount: u64,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U64(transfer_amount));
    args.push(TransactionArgument::U8Vector(vec![]));
    args.push(TransactionArgument::U8Vector(vec![]));

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        StdlibScript::PeerToPeerWithMetadata
            .compiled_bytes()
            .into_vec(),
        vec![lbr_type_tag()],
        args,
        seq_num,
        gas_costs::TXN_RESERVED, // this is a default for gas
        0,                       // this is a default for gas
        LBR_NAME.to_owned(),
    )
}

/// Returns a transaction to set config for a candidate validator
pub fn set_validator_config_txn(
    sender: &Account,
    consensus_pubkey: Vec<u8>,
    validator_network_identity_pubkey: Vec<u8>,
    validator_network_address: Vec<u8>,
    fullnodes_network_identity_pubkey: Vec<u8>,
    fullnodes_network_address: Vec<u8>,
    seq_num: u64,
) -> SignedTransaction {
    let args = vec![
        TransactionArgument::Address(*sender.address()),
        TransactionArgument::U8Vector(consensus_pubkey),
        TransactionArgument::U8Vector(validator_network_identity_pubkey),
        TransactionArgument::U8Vector(validator_network_address),
        TransactionArgument::U8Vector(fullnodes_network_identity_pubkey),
        TransactionArgument::U8Vector(fullnodes_network_address),
    ];
    sender.create_signed_txn_with_args(
        StdlibScript::SetValidatorConfig.compiled_bytes().into_vec(),
        vec![],
        args,
        seq_num,
        gas_costs::TXN_RESERVED * 3,
        0,
        LBR_NAME.to_owned(),
    )
}

/// Returns a transaction to change the keys for the given account.
pub fn rotate_key_txn(sender: &Account, new_key_hash: Vec<u8>, seq_num: u64) -> SignedTransaction {
    let args = vec![TransactionArgument::U8Vector(new_key_hash)];
    sender.create_signed_txn_with_args(
        StdlibScript::RotateAuthenticationKey
            .compiled_bytes()
            .into_vec(),
        vec![],
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        0,
        LBR_NAME.to_owned(),
    )
}

/// Returns a transaction to change the keys for the given account.
pub fn raw_rotate_key_txn(
    sender: AccountAddress,
    new_key_hash: Vec<u8>,
    seq_num: u64,
) -> RawTransaction {
    let args = vec![TransactionArgument::U8Vector(new_key_hash)];
    Account::create_raw_txn_with_args(
        sender,
        StdlibScript::RotateAuthenticationKey
            .compiled_bytes()
            .into_vec(),
        vec![],
        args,
        seq_num,
        gas_costs::TXN_RESERVED,
        0,
        LBR_NAME.to_owned(),
    )
}
