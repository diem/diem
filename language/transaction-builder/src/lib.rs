// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use compiled_stdlib::{transaction_scripts::StdlibScript, StdLibOptions};
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    on_chain_config::{LibraVersion, VMPublishingOption},
    transaction::{
        authenticator::AuthenticationKey, ChangeSet, Script, Transaction, TransactionArgument,
    },
    write_set::{WriteOp, WriteSetMut},
};
use mirai_annotations::*;
use move_core_types::language_storage::TypeTag;
use std::convert::TryFrom;
use vm::access::ModuleAccess;

/// Generated builders.
mod generated;

// Re-export all generated builders unless they are shadowed by custom builders below.
pub use generated::*;

// TODO: remove all the following compatibility aliases after migrating the codebase.
pub use crate::{
    encode_modify_publishing_option_script as encode_publishing_option_script,
    encode_update_libra_version_script as encode_update_libra_version,
};
pub use generated::{
    encode_add_recovery_rotation_capability_script as encode_add_recovery_rotation_capability,
    encode_create_child_vasp_account_script as encode_create_child_vasp_account,
    encode_create_designated_dealer_script as encode_create_designated_dealer,
    encode_create_parent_vasp_account_script as encode_create_parent_vasp_account,
    encode_create_recovery_address_script as encode_create_recovery_address,
    encode_create_validator_account_script as encode_create_validator_account,
    encode_freeze_account_script as encode_freeze_account,
    encode_peer_to_peer_with_metadata_script as encode_peer_to_peer_with_metadata,
    encode_peer_to_peer_with_metadata_script as encode_transfer_with_metadata_script,
    encode_rotate_authentication_key_with_nonce_script as encode_rotate_authentication_key_script_with_nonce,
    encode_tiered_mint_script as encode_tiered_mint,
    encode_unfreeze_account_script as encode_unfreeze_account,
    encode_update_exchange_rate_script as encode_update_exchange_rate,
    encode_update_minting_ability_script as encode_update_minting_ability,
    encode_update_travel_rule_limit_script as encode_update_travel_rule_limit,
    encode_update_unhosted_wallet_limits_script as encode_update_unhosted_wallet_limits,
};

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

fn validate_auth_key_prefix(auth_key_prefix: &[u8]) {
    let auth_key_prefix_length = auth_key_prefix.len();
    checked_assume!(
        auth_key_prefix_length == 0
            || auth_key_prefix_length == AuthenticationKey::LENGTH - AccountAddress::LENGTH,
        "Bad auth key prefix length {}",
        auth_key_prefix_length
    );
}

// TODO: this should go away once we are no longer using it in tests/testnet
/// Encode a program creating `amount` coins for sender
pub fn encode_mint_script(
    token: TypeTag,
    recipient: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    generated::encode_mint_script(token, *recipient, auth_key_prefix, amount)
}

pub fn encode_modify_publishing_option_script(config: VMPublishingOption) -> Script {
    let bytes = lcs::to_bytes(&config).expect("Cannot deserialize VMPublishingOption");
    generated::encode_modify_publishing_option_script(bytes)
}

pub fn encode_update_libra_version_script(libra_version: LibraVersion) -> Script {
    generated::encode_update_libra_version_script(libra_version.major as u64)
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
