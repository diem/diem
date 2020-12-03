// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::account::Account;
use compiled_stdlib::transaction_scripts::StdlibScript;
use compiler::Compiler;
use diem_types::{
    account_config,
    transaction::{RawTransaction, Script, SignedTransaction, TransactionArgument},
};
use move_core_types::language_storage::TypeTag;
use once_cell::sync::Lazy;

pub static CREATE_ACCOUNT_SCRIPT: Lazy<Vec<u8>> = Lazy::new(|| {
    let code = "
    import 0x1.Diem;
    import 0x1.DiemAccount;

    main<Token>(account: &signer, fresh_address: address, auth_key_prefix: vector<u8>, initial_amount: u64) {
      let with_cap: DiemAccount.WithdrawCapability;
      let name: vector<u8>;
      name = h\"\";

      DiemAccount.create_parent_vasp_account<Token>(
        copy(account),
        copy(fresh_address),
        move(auth_key_prefix),
        move(name),
        false
      );
      if (copy(initial_amount) > 0) {
         with_cap = DiemAccount.extract_withdraw_capability(copy(account));
         DiemAccount.pay_from<Token>(
           &with_cap,
           move(fresh_address),
           move(initial_amount),
           h\"\",
           h\"\"
         );
         DiemAccount.restore_withdraw_capability(move(with_cap));
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

pub fn empty_txn(
    sender: &Account,
    seq_num: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    gas_currency_code: String,
) -> SignedTransaction {
    sender
        .transaction()
        .script(Script::new(EMPTY_SCRIPT.to_vec(), vec![], vec![]))
        .sequence_number(seq_num)
        .max_gas_amount(max_gas_amount)
        .gas_unit_price(gas_unit_price)
        .gas_currency_code(&gas_currency_code)
        .sign()
}

/// Returns a transaction to create a new account with the given arguments.
pub fn create_account_txn(
    sender: &Account,
    new_account: &Account,
    seq_num: u64,
    initial_amount: u64,
    type_tag: TypeTag,
) -> SignedTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_account.address()));
    args.push(TransactionArgument::U8Vector(new_account.auth_key_prefix()));
    args.push(TransactionArgument::U64(initial_amount));

    sender
        .transaction()
        .script(Script::new(
            CREATE_ACCOUNT_SCRIPT.to_vec(),
            vec![type_tag],
            args,
        ))
        .sequence_number(seq_num)
        .sign()
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
    sender
        .transaction()
        .script(Script::new(
            StdlibScript::PeerToPeerWithMetadata
                .compiled_bytes()
                .into_vec(),
            vec![account_config::xus_tag()],
            args,
        ))
        .sequence_number(seq_num)
        .sign()
}

/// Returns a transaction to change the keys for the given account.
pub fn rotate_key_txn(sender: &Account, new_key_hash: Vec<u8>, seq_num: u64) -> SignedTransaction {
    let args = vec![TransactionArgument::U8Vector(new_key_hash)];
    sender
        .transaction()
        .script(Script::new(
            StdlibScript::RotateAuthenticationKey
                .compiled_bytes()
                .into_vec(),
            vec![],
            args,
        ))
        .sequence_number(seq_num)
        .sign()
}

/// Returns a transaction to change the keys for the given account.
pub fn raw_rotate_key_txn(sender: &Account, new_key_hash: Vec<u8>, seq_num: u64) -> RawTransaction {
    let args = vec![TransactionArgument::U8Vector(new_key_hash)];
    sender
        .transaction()
        .script(Script::new(
            StdlibScript::RotateAuthenticationKey
                .compiled_bytes()
                .into_vec(),
            vec![],
            args,
        ))
        .sequence_number(seq_num)
        .raw()
}
