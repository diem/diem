// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use compiled_stdlib::{
    stdlib_modules as compiled_stdlib_modules, transaction_scripts::StdlibScript,
};
use diem_framework::stdlib_modules as fresh_stdlib_modules;
use diem_types::{
    access_path::AccessPath,
    block_metadata::BlockMetadata,
    transaction::{ChangeSet, Transaction},
    write_set::{WriteOp, WriteSetMut},
};
use std::convert::TryFrom;

// TODO: this should go away once we are no longer using it in tests
pub fn encode_block_prologue_script(block_metadata: BlockMetadata) -> Transaction {
    Transaction::BlockMetadata(block_metadata)
}

/// Update WriteSet
pub fn encode_stdlib_upgrade_transaction(use_fresh_modules: bool) -> ChangeSet {
    let mut write_set = WriteSetMut::new(vec![]);
    let stdlib_modules = if use_fresh_modules {
        fresh_stdlib_modules()
    } else {
        compiled_stdlib_modules()
    };
    for (bytes, module) in stdlib_modules.bytes_and_modules() {
        write_set.push((
            AccessPath::code_access_path(module.self_id()),
            WriteOp::Value(bytes.clone()),
        ));
    }
    ChangeSet::new(
        write_set.freeze().expect("Failed to create writeset"),
        vec![],
    )
}

// TODO: delete and use StdlibScript::try_from directly if it's ok to drop the "_transaction"?
/// Returns a user friendly mnemonic for the transaction type if the transaction is
/// for a known, white listed, transaction.
pub fn get_transaction_name(code: &[u8]) -> String {
    StdlibScript::try_from(code).map_or("<unknown transaction>".to_string(), |name| {
        format!("{}_transaction", name)
    })
}
