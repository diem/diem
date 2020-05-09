// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

#[cfg(any(test, feature = "fuzzing"))]
use libra_types::account_config;
use libra_types::{
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    on_chain_config::{LibraVersion, VMPublishingOption},
    transaction::{authenticator::AuthenticationKey, Script, Transaction, TransactionArgument},
};
use mirai_annotations::*;
use move_core_types::language_storage::TypeTag;
use std::convert::TryFrom;
use stdlib::transaction_scripts::StdlibScript;
#[cfg(any(test, feature = "fuzzing"))]
use vm::file_format::{Bytecode, CompiledScript};

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
    (name: $name:ident,
     type_arg: $ty_arg_name:ident,
     args: [$($arg_name:ident: $arg_ty:ident),*],
     script: $script_name:ident,
     doc: $comment:literal
    ) => {
        #[doc=$comment]
        pub fn $name($ty_arg_name: TypeTag, $($arg_name: to_rust_ty!($arg_ty),)*) -> Script {
            encode_txn_script!([$ty_arg_name], [$($arg_name: $arg_ty),*], $script_name)
        }
    };
    (name: $name:ident,
     args: [$($arg_name:ident: $arg_ty:ident),*],
     script: $script_name:ident,
     doc: $comment:literal
    ) => {
        #[doc=$comment]
        pub fn $name($($arg_name: to_rust_ty!($arg_ty),)*) -> Script {
            encode_txn_script!([], [$($arg_name: $arg_ty),*], $script_name)
        }
    };
    ([$($ty_arg_name:ident),*],
     [$($arg_name:ident: $arg_ty:ident),*],
     $script_name:ident
    ) => {
            Script::new(
                StdlibScript::$script_name.compiled_bytes().into_vec(),
                vec![$($ty_arg_name),*],
                vec![
                    $(to_txn_arg!($arg_ty, $arg_name),)*
                ],
            )
    };
}

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

encode_txn_script! {
    name: encode_add_validator_script,
    args: [new_validator: Address],
    script: AddValidator,
    doc: "Encode a program adding `new_validator` to the pending validator set. Fails if the\
          `new_validator` address is already in the validator set, already in the pending validator set,\
          or does not have a `ValidatorConfig` resource stored at the address"
}

encode_txn_script! {
    name: encode_burn_script,
    type_arg: type_,
    args: [preburn_address: Address],
    script: Burn,
    doc: "Permanently destroy the coins stored in the oldest burn request under the `Preburn`\
          resource stored at `preburn_address`. This will only succeed if the sender has a\
          `MintCapability` stored under their account and `preburn_address` has a pending burn request"
}

encode_txn_script! {
    name: encode_cancel_burn_script,
    type_arg: type_,
    args: [preburn_address: Address],
    script: CancelBurn,
    doc: "Cancel the oldest burn request from `preburn_address` and return the funds to\
          `preburn_address`.  Fails if the sender does not have a published `MintCapability`."
}

/// Encode a program transferring `amount` coins from `sender` to `recipient` with (optional)
/// associated metadata `metadata` and (optional) `signature` on the metadata.
/// The `metadata` and `signature` parameters are only required if `amount` >= 1000 LBR and the
/// sender and recipient of the funds are two distinct VASPs.
/// Fails if there is no account at the recipient address or if the sender's balance is lower than
/// `amount`.
pub fn encode_transfer_with_metadata_script(
    type_: TypeTag,
    recipient: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
    metadata: Vec<u8>,
    signature: Vec<u8>,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    Script::new(
        StdlibScript::PeerToPeerWithMetadata
            .compiled_bytes()
            .into_vec(),
        vec![type_],
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
            TransactionArgument::U8Vector(metadata),
            TransactionArgument::U8Vector(signature),
        ],
    )
}

/// Encode a program transferring `amount` coins from `sender` to `recipient` but pad the output
/// bytecode with unreachable instructions.
#[cfg(any(test, feature = "fuzzing"))]
pub fn encode_transfer_script_with_padding(
    recipient: &AccountAddress,
    amount: u64,
    padding_size: u64,
) -> Script {
    let mut script_mut =
        CompiledScript::deserialize(&StdlibScript::PeerToPeer.compiled_bytes().into_vec())
            .unwrap()
            .into_inner();
    script_mut
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
        vec![account_config::lbr_type_tag()],
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U8Vector(vec![]), // use empty auth key prefix
            TransactionArgument::U64(amount),
        ],
    )
}

encode_txn_script! {
    name: encode_preburn_script,
    type_arg: type_,
    args: [amount: U64],
    script: Preburn,
    doc: "Preburn `amount` coins from the sender's account. This will only succeed if the sender\
          already has a published `Preburn` resource."
}

/// Encode a program creating a fresh account at `account_address` with `initial_balance` coins
/// transferred from the sender's account balance. Fails if there is already an account at
/// `account_address` or if the sender's balance is lower than `initial_balance`.
pub fn encode_create_account_script(
    token: TypeTag,
    account_address: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    initial_balance: u64,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    Script::new(
        StdlibScript::CreateAccount.compiled_bytes().into_vec(),
        vec![token],
        vec![
            TransactionArgument::Address(*account_address),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(initial_balance),
        ],
    )
}

encode_txn_script! {
    name: encode_publish_shared_ed25519_public_key_script,
    args: [public_key: Bytes],
    script: PublishSharedEd2551PublicKey,
    doc: "(1) Rotate the authentication key of the sender to `public_key`\
          (2) Publish a resource containing a 32-byte ed25519 public key and the rotation capability\
          of the sender under the sender's address.\
          Aborts if the sender already has a `SharedEd25519PublicKey` resource.\
          Aborts if the length of `new_public_key` is not 32."
}

encode_txn_script! {
    name: encode_add_currency_to_account_script,
    type_arg: currency,
    args: [],
    script: AddCurrencyToAccount,
    doc: "Add the currency identified by the type `currency` to the sending accounts.\
          Aborts if the account already holds a balance fo `currency` type."
}

encode_txn_script! {
    name: encode_register_preburner_script,
    type_arg: type_,
    args: [],
    script: RegisterPreburner,
    doc: "Publish a newly created `Preburn` resource under the sender's account.\
          This will fail if the sender already has a published `Preburn` resource."
}

encode_txn_script! {
    name: encode_register_validator_script,
    args: [
        consensus_pubkey: Bytes,
        validator_network_signing_pubkey: Bytes,
        validator_network_identity_pubkey: Bytes,
        validator_network_address: Bytes,
        fullnodes_network_identity_pubkey: Bytes,
        fullnodes_network_address: Bytes
    ],
    script: RegisterValidator,
    doc: "Encode a program registering the sender as a candidate validator with the given key information.\
         `network_signing_pubkey` should be a Ed25519 public key\
         `network_identity_pubkey` should be a X25519 public key\
         `consensus_pubkey` should be a Ed25519 public c=key."
}

encode_txn_script! {
    name: encode_remove_validator_script,
    args: [to_remove: Address],
    script: RemoveValidator,
    doc: "Encode a program adding `to_remove` to the set of pending validator removals. Fails if\
          the `to_remove` address is already in the validator set or already in the pending removals."
}

encode_txn_script! {
    name: encode_rotate_compliance_public_key_script,
    args: [vasp_root_addr: Address, new_key: Bytes],
    script: RotateCompliancePublicKey,
    doc: "Encode a program that rotates `vasp_root_addr`'s compliance public key to `new_key`."
}

encode_txn_script! {
    name: encode_rotate_consensus_pubkey_script,
    args: [new_key: Bytes],
    script: RotateConsensusPubkey,
    doc: "Encode a program that rotates the sender's consensus public key to `new_key`."
}

encode_txn_script! {
    name: rotate_authentication_key_script,
    args: [new_hashed_key: Bytes],
    script: RotateAuthenticationKey,
    doc: "Encode a program that rotates the sender's authentication key to `new_key`. `new_key`\
          should be a 256 bit sha3 hash of an ed25519 public key."
}

encode_txn_script! {
    name: encode_rotate_shared_ed25519_public_key_script,
    args: [new_public_key: Bytes],
    script: RotateSharedEd2551PublicKey,
    doc: "(1) rotate the public key stored in the sender's `SharedEd25519PublicKey` resource to\
          `new_public_key`\
          (2) rotate the authentication key using the capability stored in the sender's\
          `SharedEd25519PublicKey` to a new value derived from `new_public_key`\
          Aborts if the sender does not have a `SharedEd25519PublicKey` resource.\
          Aborts if the length of `new_public_key` is not 32."
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
    Script::new(
        StdlibScript::Mint.compiled_bytes().into_vec(),
        vec![token],
        vec![
            TransactionArgument::Address(*sender),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
        ],
    )
}

/// Encode a program creating `amount` LBR for `address`
pub fn encode_mint_lbr_to_address_script(
    address: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
) -> Script {
    validate_auth_key_prefix(&auth_key_prefix);
    Script::new(
        StdlibScript::MintLbrToAddress.compiled_bytes().into_vec(),
        vec![],
        vec![
            TransactionArgument::Address(*address),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
        ],
    )
}

pub fn encode_publishing_option_script(config: VMPublishingOption) -> Script {
    let bytes = lcs::to_bytes(&config).expect("Cannot deserialize VMPublishingOption");
    Script::new(
        StdlibScript::ModifyPublishingOption
            .compiled_bytes()
            .into_vec(),
        vec![],
        vec![TransactionArgument::U8Vector(bytes)],
    )
}

pub fn encode_update_libra_version(libra_version: LibraVersion) -> Script {
    Script::new(
        StdlibScript::UpdateLibraVersion.compiled_bytes().into_vec(),
        vec![],
        vec![TransactionArgument::U64(libra_version.major as u64)],
    )
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
// on-chain LBR scripts
//...........................................................................

encode_txn_script! {
    name: encode_mint_lbr,
    args: [amount_lbr: U64],
    script: MintLbr,
    doc: "Mints `amount_lbr` LBR from the sending account's constituent coins and deposits the\
          resulting LBR into the sending account."
}

encode_txn_script! {
    name: encode_unmint_lbr,
    args: [amount_lbr: U64],
    script: UnmintLbr,
    doc: "Unmints `amount_lbr` LBR from the sending account into the constituent coins and deposits\
          the resulting coins into the sending account."
}

//...........................................................................
//  Association-related scripts
//...........................................................................

encode_txn_script! {
    name: encode_update_exchange_rate,
    type_arg: currency,
    args: [new_exchange_rate_denominator: U64, new_exchange_rate_numerator: U64],
    script: UpdateExchangeRate,
    doc: "Updates the on-chain exchange rate to LBR for the given `currency` to be given by\
         `new_exchange_rate_denominator/new_exchange_rate_numerator`."
}

encode_txn_script! {
    name: encode_update_minting_ability,
    type_arg: currency,
    args: [allow_minting: Bool],
    script: UpdateMintingAbility,
    doc: "Allows--true--or disallows--false--minting of `currency` based upon `allow_minting`."
}

//...........................................................................
// VASP-related scripts
//...........................................................................

encode_txn_script! {
    name: create_parent_vasp_account,
    args: [address: Address, auth_key_prefix: Bytes, human_name: Bytes, base_url: Bytes, compliance_public_key: Bytes],
    script: CreateParentVaspAccount,
    doc: "Create an account with the ParentVASP role at `address` with authentication key\
          `auth_key_prefix` | `new_account_address`."
}

encode_txn_script! {
    name: create_child_vasp_account,
    args: [address: Address, auth_key_prefix: Bytes],
    script: CreateChildVaspAccount,
    doc: "Create an account with the ChildVASP role at `address` with authentication key\
          `auth_key_prefix` | `new_account_address`. This account will be a child of the\
          transaction sender, which must be a ParentVASP."
}

encode_txn_script! {
    name: encode_freeze_account,
    args: [addr: Address],
    script: FreezeAccount,
    doc: "Freezes account with address addr."
}

encode_txn_script! {
    name: encode_unfreeze_account,
    args: [addr: Address],
    script: UnfreezeAccount,
    doc: "Unfreezes account with address addr."
}
