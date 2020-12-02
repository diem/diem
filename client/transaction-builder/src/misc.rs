// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use compiled_stdlib::{transaction_scripts::StdlibScript, StdLibOptions};
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
pub fn encode_stdlib_upgrade_transaction(option: StdLibOptions) -> ChangeSet {
    let mut write_set = WriteSetMut::new(vec![]);
    let stdlib = compiled_stdlib::stdlib_modules(option);
    for module in stdlib {
        let mut bytes = vec![];
        module
            .serialize(&mut bytes)
            .expect("Failed to serialize module");
        write_set.push((
            AccessPath::code_access_path(module.self_id()),
            WriteOp::Value(bytes),
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
