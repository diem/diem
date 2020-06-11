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
        authenticator::AuthenticationKey, ArgumentABI, ChangeSet, Script, ScriptABI, Transaction,
        TransactionArgument, TypeArgumentABI,
    },
    write_set::{WriteOp, WriteSetMut},
};
use mirai_annotations::*;
use move_core_types::language_storage::TypeTag;
use std::convert::TryFrom;
use vm::access::ModuleAccess;

fn validate_auth_key_prefix(auth_key_prefix: &[u8]) {
    let auth_key_prefix_length = auth_key_prefix.len();
    checked_assume!(
        auth_key_prefix_length == 0
            || auth_key_prefix_length == AuthenticationKey::LENGTH - AccountAddress::LENGTH,
        "Bad auth key prefix length {}",
        auth_key_prefix_length
    );
}

macro_rules! to_rust_ty {
    (U64) => { u64 };
    (Address) => { AccountAddress };
    (Bytes) => { Vec<u8> };
    (Bool) => { bool };
}

macro_rules! to_type_tag {
    (U64) => {
        TypeTag::U64
    };
    (Address) => {
        TypeTag::Address
    };
    (Bytes) => {
        TypeTag::Vector(Box::new(TypeTag::U8))
    };
    (Bool) => {
        TypeTag::Bool
    };
}

macro_rules! to_txn_arg {
    (U64, $param_name: ident) => {
        TransactionArgument::U64($param_name)
    };
    (Address, $param_name: ident) => {
        TransactionArgument::Address($param_name)
    };
    (Bytes, $param_name: ident) => {
        TransactionArgument::U8Vector($param_name)
    };
    (Bool, $param_name: ident) => {
        TransactionArgument::Bool($param_name)
    };
}

macro_rules! encode_txn_script {
    (builder: $name:ident,
     type_args: [$($ty_arg_name:ident),*],
     args: [$($arg_name:ident: $arg_ty:ident),*],
     script: $script_name:ident,
     doc: $comment:literal
    ) => {
        #[doc=$comment]
        pub fn $name($($ty_arg_name: TypeTag,)* $($arg_name: to_rust_ty!($arg_ty),)*) -> Script {
            Script::new(
                StdlibScript::$script_name.compiled_bytes().into_vec(),
                vec![$($ty_arg_name),*],
                vec![
                    $(to_txn_arg!($arg_ty, $arg_name),)*
                ],
            )
        }
    };
}

macro_rules! to_script_abi {
    (script: $script_name:ident,
     type_args: [$($ty_arg_name:ident),*],
     args: [$($arg_name:ident: $arg_ty:ident),*],
     doc: $comment:literal
    ) => {
        ScriptABI::new(
            StdlibScript::$script_name.name(),
            $comment.to_string(),
            StdlibScript::$script_name.compiled_bytes().into_vec(),
            vec![$(TypeArgumentABI::new(stringify!($ty_arg_name).to_string()),)*],
            vec![$(ArgumentABI::new(stringify!($arg_name).to_string(), to_type_tag!($arg_ty)),)*],
        )
    };
}

macro_rules! register_all_txn_scripts {
    ($({
        script: $script_name:ident,
        builder: $builder_name:ident,
        type_args: [$($ty_arg_name:ident),*],
        args: [$($arg_name:ident: $arg_ty:ident),*],
        doc: $comment:literal
    })*) => {
        $(
            encode_txn_script! {
                builder: $builder_name,
                type_args: [$($ty_arg_name),*],
                args: [$($arg_name: $arg_ty),*],
                script: $script_name,
                doc: $comment
            }
        )*

        pub fn get_stdlib_script_abis() -> Vec<ScriptABI> {
            vec![$(
                to_script_abi! {
                    script: $script_name,
                    type_args: [$($ty_arg_name),*],
                    args: [$($arg_name: $arg_ty),*],
                    doc: $comment
                },
            )*]
        }
    }
}

register_all_txn_scripts! {

//...........................................................................
// Main scripts
//...........................................................................

{
    script: AddRecoveryRotationCapability,
    builder: encode_add_recovery_rotation_capability,
    type_args: [],
    args: [recovery_address: Address],
    doc: "Add the `KeyRotationCapability` for `to_recover_account` to the `RecoveryAddress`\
          resource under `recovery_address`. Aborts if `to_recovery_account` and\
          `to_recovery_address belong to different VASPs, if `recovery_address` does not have a\
          `RecoveryAddress` resource, or if `to_recover_account` has already extracted its\
          `KeyRotationCapability`."
}

{
    script: AddValidator,
    builder: encode_add_validator_script,
    type_args: [],
    args: [new_validator: Address],
    doc: "Encode a program adding `new_validator` to the pending validator set. Fails if the\
          `new_validator` address is already in the validator set, already in the pending validator set,\
          or does not have a `ValidatorConfig` resource stored at the address"
}

{
    script: Burn,
    builder: encode_burn_script,
    type_args: [type_],
    args: [nonce: U64, preburn_address: Address],
    doc: "Permanently destroy the coins stored in the oldest burn request under the `Preburn`\
          resource stored at `preburn_address`. This will only succeed if the sender has a\
          `MintCapability` stored under their account and `preburn_address` has a pending burn request"
}

{
    script: BurnTxnFees,
    builder: encode_burn_txn_fees_script,
    type_args: [currency],
    args: [],
    doc: "Burn transaction fees that have been collected in the given `currency`,\
          and relinquish to the association. The currency must be non-synthetic."
}

{
    script: CancelBurn,
    builder: encode_cancel_burn_script,
    type_args: [type_],
    args: [preburn_address: Address],
    doc: "Cancel the oldest burn request from `preburn_address` and return the funds to\
          `preburn_address`.  Fails if the sender does not have a published `MintCapability`."
}

{
    script: CreateRecoveryAddress,
    builder: encode_create_recovery_address,
    type_args: [],
    args: [],
    doc: "Extract the `KeyRotationCapability` for `recovery_account` and publish it in a\
          `RecoveryAddress` resource under  `recovery_account`. Aborts if `recovery_account` has\
           delegated its `KeyRotationCapability`, already has a`RecoveryAddress` resource, or is\
           not a VASP. Cancel the oldest burn request from `preburn_address` and return the funds\
           to `preburn_address`.  Fails if the sender does not have a published `MintCapability`."
}

{
    script: PeerToPeerWithMetadata,
    builder: encode_transfer_with_metadata_script,
    type_args: [coin_type],
    args: [recipient_address: Address, amount: U64, metadata: Bytes, metadata_signature: Bytes],
    doc: "Encode a program transferring `amount` coins to `recipient_address` with (optional)\
          associated metadata `metadata` and (optional) `signature` on the metadata, amount, and\
          sender address. The `metadata` and `signature` parameters are only required if\
          `amount` >= 1000 LBR and the sender and recipient of the funds are two distinct VASPs.\
          Fails if there is no account at the recipient address or if the sender's balance is lower\
          than `amount`"
}

{
    script: Preburn,
    builder: encode_preburn_script,
    type_args: [type_],
    args: [amount: U64],
    doc: "Preburn `amount` coins from the sender's account. This will only succeed if the sender\
          already has a published `Preburn` resource."
}

{
    script: PublishSharedEd2551PublicKey,
    builder: encode_publish_shared_ed25519_public_key_script,
    type_args: [],
    args: [public_key: Bytes],
    doc: "(1) Rotate the authentication key of the sender to `public_key`\
          (2) Publish a resource containing a 32-byte ed25519 public key and the rotation capability\
          of the sender under the sender's address.\
          Aborts if the sender already has a `SharedEd25519PublicKey` resource.\
          Aborts if the length of `new_public_key` is not 32."
}

{
    script: AddCurrencyToAccount,
    builder: encode_add_currency_to_account_script,
    type_args: [currency],
    args: [],
    doc: "Add the currency identified by the type `currency` to the sending accounts.\
          Aborts if the account already holds a balance fo `currency` type."
}

{
    script: RegisterPreburner,
    builder: encode_register_preburner_script,
    type_args: [type_],
    args: [],
    doc: "Publish a newly created `Preburn` resource under the sender's account.\
          This will fail if the sender already has a published `Preburn` resource."
}

{
    script: RegisterValidator,
    builder: encode_register_validator_script,
    type_args: [],
    args: [
        consensus_pubkey: Bytes,
        validator_network_identity_pubkey: Bytes,
        validator_network_address: Bytes,
        fullnodes_network_identity_pubkey: Bytes,
        fullnodes_network_address: Bytes
    ],
    doc: "Encode a program registering the sender as a candidate validator with the given key information.\
         `network_identity_pubkey` should be a X25519 public key\
         `consensus_pubkey` should be a Ed25519 public c=key."
}

{
    script: RemoveValidator,
    builder: encode_remove_validator_script,
    type_args: [],
    args: [to_remove: Address],
    doc: "Encode a program adding `to_remove` to the set of pending validator removals. Fails if\
          the `to_remove` address is already in the validator set or already in the pending removals."
}

{
    script: RotateCompliancePublicKey,
    builder: encode_rotate_compliance_public_key_script,
    type_args: [],
    args: [new_key: Bytes],
    doc: "Encode a program that rotates `vasp_root_addr`'s compliance public key to `new_key`."
}

{
    script: RotateBaseUrl,
    builder: encode_rotate_base_url_script,
    type_args: [],
    args: [new_url: Bytes],
    doc: "Encode a program that rotates `vasp_root_addr`'s base URL to `new_url`."
}

{
    script: RotateConsensusPubkey,
    builder: encode_rotate_consensus_pubkey_script,
    type_args: [],
    args: [new_key: Bytes],
    doc: "Encode a program that rotates the sender's consensus public key to `new_key`."
}

{
    script: RotateAuthenticationKey,
    builder: encode_rotate_authentication_key_script,
    type_args: [],
    args: [new_hashed_key: Bytes],
    doc: "Encode a program that rotates the sender's authentication key to `new_key`. `new_key`\
          should be a 256 bit sha3 hash of an ed25519 public key."
}

{
    script: RotateAuthenticationKeyWithRecoveryAddress,
    builder: encode_rotate_authentication_key_with_recovery_address_script,
    type_args: [],
    args: [recovery_address: Address, to_recover: Address, new_key: Bytes],
    doc: "Rotate the authentication key of `to_recover` to `new_key`. Can be invoked by either\
          `recovery_address` or `to_recover`. Aborts if `recovery_address` does not have the\
          `KeyRotationCapability` for `to_recover`."
}

{
    script: RotateSharedEd2551PublicKey,
    builder: encode_rotate_shared_ed25519_public_key_script,
    type_args: [],
    args: [new_public_key: Bytes],
    doc: "(1) rotate the public key stored in the sender's `SharedEd25519PublicKey` resource to\
          `new_public_key`\
          (2) rotate the authentication key using the capability stored in the sender's\
          `SharedEd25519PublicKey` to a new value derived from `new_public_key`\
          Aborts if the sender does not have a `SharedEd25519PublicKey` resource.\
          Aborts if the length of `new_public_key` is not 32."
}

{
    script: Mint,
    builder: _encode_mint_script_internal,
    type_args: [token],
    args: [sender: Address, auth_key_prefix: Bytes, amount: U64],
    doc: "Encode a program creating `amount` coins for sender"
}

{
    script: MintLbrToAddress,
    builder: _encode_mint_lbr_to_address_script_internal,
    type_args: [],
    args: [address: Address, auth_key_prefix: Bytes, amount: U64],
    doc: "Encode a program creating `amount` LBR for `address`"
}

{
    script: ModifyPublishingOption,
    builder: _encode_publishing_option_script_internal,
    type_args: [],
    args: [config_bytes: Bytes],
    doc: "Modify publishing options. Takes the LCS bytes of a `VMPublishingOption` object as input."
}

{
    script: UpdateLibraVersion,
    builder: _encode_update_libra_version_internal,
    type_args: [],
    args: [libra_version_major: U64],
    doc: "Update Libra version"
}

//...........................................................................
// on-chain LBR scripts
//...........................................................................

{
    script: MintLbr,
    builder: encode_mint_lbr,
    type_args: [],
    args: [amount_lbr: U64],
    doc: "Mints `amount_lbr` LBR from the sending account's constituent coins and deposits the\
          resulting LBR into the sending account."
}

{
    script: UnmintLbr,
    builder: encode_unmint_lbr,
    type_args: [],
    args: [amount_lbr: U64],
    doc: "Unmints `amount_lbr` LBR from the sending account into the constituent coins and deposits\
          the resulting coins into the sending account."
}

//...........................................................................
//  Association-related scripts
//...........................................................................


{
    script: UpdateMintingAbility,
    builder: encode_update_minting_ability,
    type_args: [currency],
    args: [allow_minting: Bool],
    doc: "Allows--true--or disallows--false--minting of `currency` based upon `allow_minting`."
}

//...........................................................................
// VASP-related scripts
//...........................................................................

{
    script: CreateParentVaspAccount,
    builder: encode_create_parent_vasp_account,
    type_args: [currency],
    args: [address: Address, auth_key_prefix: Bytes, human_name: Bytes, base_url: Bytes, compliance_public_key: Bytes, add_all_currencies: Bool],
    doc: "Create an account with the ParentVASP role at `address` with authentication key\
          `auth_key_prefix` | `new_account_address` and a 0 balance of type `currency`. If\
          `add_all_currencies` is true, 0 balances for all available currencies in the system will\
          also be added. This can only be invoked by an Association account."
}

{
    script: CreateChildVaspAccount,
    builder: encode_create_child_vasp_account,
    type_args: [currency],
    args: [address: Address, auth_key_prefix: Bytes, add_all_currencies: Bool, initial_balance: U64],
    doc: "Create an account with the ChildVASP role at `address` with authentication key\
          `auth_key_prefix` | `new_account_address` and `initial_balance` of type `currency`\
          transferred from the sender. If `add_all_currencies` is true, 0 balances for all\
          available currencies in the system will also be added to the account. This account will\
          be a child of the transaction sender, which must be a ParentVASP."
}

//...........................................................................
// Treasury Compliance Scripts
//...........................................................................

{
    script: TieredMint,
    builder: encode_tiered_mint,
    type_args: [coin_type],
    args: [nonce: U64, designated_dealer_address: Address, mint_amount: U64, tier_index: U64],
    doc: "Mints 'mint_amount' to 'designated_dealer_address' for 'tier_index' tier.\
          Max valid tier index is 3 since there are max 4 tiers per DD.
          Sender should be treasury compliance account and receiver authorized DD"
}

{
    script: CreateDesignatedDealer,
    builder: encode_create_designated_dealer,
    type_args: [coin_type],
    args: [nonce: U64, new_account_address: Address, auth_key_prefix: Bytes],
    doc: "Creates designated dealer at 'new_account_address"
}

{
    script: FreezeAccount,
    builder: encode_freeze_account,
    type_args: [],
    args: [nonce: U64, addr: Address],
    doc: "Freezes account with address addr."
}

{
    script: UnfreezeAccount,
    builder: encode_unfreeze_account,
    type_args: [],
    args: [nonce: U64, addr: Address],
    doc: "Unfreezes account with address addr."
}

{
    script: RotateAuthenticationKeyWithNonce,
    builder: encode_rotate_authentication_key_script_with_nonce,
    type_args: [],
    args: [nonce: U64, new_hashed_key: Bytes],
    doc: "Encode a program that rotates the sender's authentication key to `new_key`. `new_key`\
          should be a 256 bit sha3 hash of an ed25519 public key. This script also takes nonce"
}

{
    script: UpdateExchangeRate,
    builder: encode_update_exchange_rate,
    type_args: [currency],
    args: [nonce: U64, new_exchange_rate_denominator: U64, new_exchange_rate_numerator: U64],
    doc: "Updates the on-chain exchange rate to LBR for the given `currency` to be given by\
         `new_exchange_rate_denominator/new_exchange_rate_numerator`."
}

} // End of txn scripts

//...........................................................................
// Custom builders
//...........................................................................

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

// TODO: this should go away once we are no longer using it in tests
/// Encode a program creating `amount` coins for sender
pub fn encode_mint_script(
    token: TypeTag,
    sender: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    _encode_mint_script_internal(token, *sender, auth_key_prefix, amount)
}

/// Encode a program creating `amount` LBR for `address`
pub fn encode_mint_lbr_to_address_script(
    address: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    _encode_mint_lbr_to_address_script_internal(*address, auth_key_prefix, amount)
}

pub fn encode_publishing_option_script(config: VMPublishingOption) -> Script {
    let bytes = lcs::to_bytes(&config).expect("Cannot deserialize VMPublishingOption");
    _encode_publishing_option_script_internal(bytes)
}

pub fn encode_update_libra_version(libra_version: LibraVersion) -> Script {
    _encode_update_libra_version_internal(libra_version.major as u64)
}

// TODO: this should go away once we are no longer using it in tests
pub fn encode_block_prologue_script(block_metadata: BlockMetadata) -> Transaction {
    Transaction::BlockMetadata(block_metadata)
}

// TODO: delete and use StdlibScript::try_from directly if it's ok to drop the "_transaction"?
/// Returns a user friendly mnemonic for the transaction type if the transaction is
/// for a known, white listed, transaction.
pub fn get_transaction_name(code: &[u8]) -> String {
    StdlibScript::try_from(code).map_or("<unknown transaction>".to_string(), |name| {
        format!("{}_transaction", name)
    })
}

//...........................................................................
// WriteSets
//...........................................................................

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

#[cfg(test)]
mod tests {

    #[test]
    fn test_abis() {
        let abis = super::get_stdlib_script_abis();
        assert_eq!(
            vec![
                "add_recovery_rotation_capability",
                "add_validator",
                "burn",
                "burn_txn_fees",
                "cancel_burn",
                "create_recovery_address",
                "peer_to_peer_with_metadata",
                "preburn",
                "publish_shared_ed25519_public_key",
                "add_currency_to_account",
                "register_preburner",
                "register_validator",
                "remove_validator",
                "rotate_compliance_public_key",
                "rotate_base_url",
                "rotate_consensus_pubkey",
                "rotate_authentication_key",
                "rotate_authentication_key_with_recovery_address",
                "rotate_shared_ed25519_public_key",
                "mint",
                "mint_lbr_to_address",
                "modify_publishing_option",
                "update_libra_version",
                "mint_lbr",
                "unmint_lbr",
                "update_minting_ability",
                "create_parent_vasp_account",
                "create_child_vasp_account",
                "tiered_mint",
                "create_designated_dealer",
                "freeze_account",
                "unfreeze_account",
                "rotate_authentication_key_with_nonce",
                "update_exchange_rate",
            ],
            abis.iter().map(|x| x.name()).collect::<Vec<_>>()
        );
    }
}
