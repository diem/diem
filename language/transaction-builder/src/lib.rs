// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use compiled_stdlib::{transaction_scripts::StdlibScript, StdLibOptions};
use libra_types::{
    access_path::AccessPath,
    block_metadata::BlockMetadata,
    on_chain_config::{LibraVersion, VMPublishingOption},
    transaction::{ChangeSet, Script, Transaction, TransactionArgument},
    write_set::{WriteOp, WriteSetMut},
};
use move_core_types::language_storage::TypeTag;
use std::convert::TryFrom;
pub use transaction_builder_generated::stdlib::*;

/// Encode `stdlib_script` with arguments `args`.
/// Note: this is not type-safe; the individual type-safe wrappers below should be used when
/// possible.
pub fn encode_stdlib_script(
    stdlib_script: StdlibScript,
    type_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
) -> Script {
    Script::new(stdlib_script.compiled_bytes().into_vec(), type_args, args)
}

pub fn encode_modify_publishing_option_script(config: VMPublishingOption) -> Script {
    let bytes = lcs::to_bytes(&config).expect("Cannot deserialize VMPublishingOption");
    transaction_builder_generated::stdlib::encode_modify_publishing_option_script(bytes)
}

pub fn encode_update_libra_version_script(libra_version: LibraVersion) -> Script {
    transaction_builder_generated::stdlib::encode_update_libra_version_script(
        libra_version.major as u64,
    )
}

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
            AccessPath::code_access_path(&module.self_id()),
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
