// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Rust representation of a Move transaction script that can be executed on the Libra blockchain.
//! Libra does not allow arbitrary transaction scripts; only scripts whose hashes are present in
//! the on-chain script whitelist. The genesis whitelist is derived from this file, and the
//! `Stdlib` script enum will be modified to reflect changes in the on-chain whitelist as time goes
//! on.

use anyhow::{anyhow, Error, Result};
use include_dir::{include_dir, Dir};
use libra_crypto::HashValue;
use libra_types::transaction::SCRIPT_HASH_LENGTH;
use std::{convert::TryFrom, fmt, path::PathBuf};

// This includes the compiled transaction scripts as binaries. We must use this hack to work around
// a problem with Docker, which does not copy over the Move source files that would be be used to
// produce these binaries at runtime.
#[allow(dead_code)]
const STAGED_TXN_SCRIPTS_DIR: Dir = include_dir!("staged/transaction_scripts");

/// All of the Move transaction scripts that can be executed on the Libra blockchain
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum StdlibScript {
    AddValidator,
    AddCurrency,
    AddCurrencyToAccount,
    AllowChildAccounts,
    ApplyForAssociationAddress,
    ApplyForAssociationPrivilege,
    ApplyForChildVaspCredential,
    ApplyForParentCapability,
    ApplyForRootVasp,
    ApprovedPayment,
    Burn,
    CancelBurn,
    CreateAccount,
    CreateEmptyAccount,
    EmptyScript,
    GrantAssociationAddress,
    GrantAssociationPrivilege,
    GrantChildAccount,
    GrantParentAccount,
    GrantVaspAccount,
    Mint,
    MintLbr,
    ModifyPublishingOption,
    PeerToPeer,
    PeerToPeerWithMetadata,
    Preburn,
    PublishSharedEd2551PublicKey,
    RecertifyChildAccount,
    RegisterApprovedPayment,
    RegisterPreburner,
    RegisterValidator,
    RemoveAssociationAddress,
    RemoveAssociationPrivilege,
    RemoveChildAccount,
    RemoveParentAccount,
    RemoveValidator,
    RotateAuthenticationKey,
    RotateConsensusPubkey,
    RotateSharedEd2551PublicKey,
    UnmintLbr,
    UpdateLibraVersion,
    UpdateExchangeRate,
    UpdateMintingAbility,
    // ...add new scripts here
}

impl StdlibScript {
    /// Return a vector containing all of the standard library scripts (i.e., all inhabitants of the
    /// StdlibScript enum)
    pub fn all() -> Vec<Self> {
        use StdlibScript::*;
        vec![
            AddValidator,
            AddCurrency,
            AddCurrencyToAccount,
            AllowChildAccounts,
            ApplyForAssociationAddress,
            ApplyForAssociationPrivilege,
            ApplyForChildVaspCredential,
            ApplyForParentCapability,
            ApplyForRootVasp,
            ApprovedPayment,
            Burn,
            CancelBurn,
            CreateAccount,
            CreateEmptyAccount,
            EmptyScript,
            GrantAssociationAddress,
            GrantAssociationPrivilege,
            GrantChildAccount,
            GrantParentAccount,
            GrantVaspAccount,
            Mint,
            MintLbr,
            ModifyPublishingOption,
            PeerToPeer,
            PeerToPeerWithMetadata,
            Preburn,
            PublishSharedEd2551PublicKey,
            RecertifyChildAccount,
            RegisterApprovedPayment,
            RegisterPreburner,
            RegisterValidator,
            RemoveAssociationAddress,
            RemoveAssociationPrivilege,
            RemoveChildAccount,
            RemoveParentAccount,
            RemoveValidator,
            RotateAuthenticationKey,
            RotateConsensusPubkey,
            RotateSharedEd2551PublicKey,
            UnmintLbr,
            UpdateLibraVersion,
            UpdateExchangeRate,
            UpdateMintingAbility,
            // ...add new scripts here
        ]
    }

    /// Construct the whitelist of script hashes used to determine whether a transaction script can
    /// be executed on the Libra blockchain
    pub fn whitelist() -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
        StdlibScript::all()
            .iter()
            .map(|script| *script.compiled_bytes().hash().as_ref())
            .collect()
    }

    /// Return a lowercase-underscore style name for this script
    pub fn name(self) -> String {
        self.to_string()
    }

    /// Return true if `code_bytes` is the bytecode of one of the standard library scripts
    pub fn is(code_bytes: &[u8]) -> bool {
        Self::try_from(code_bytes).is_ok()
    }

    /// Return the Move bytecode produced by compiling this script. This will almost always read
    /// from disk rather invoking the compiler; genesis is the only exception.
    pub fn compiled_bytes(self) -> CompiledBytes {
        // read from disk
        let mut path = PathBuf::from(self.name());
        path.set_extension("mv");
        CompiledBytes(
            STAGED_TXN_SCRIPTS_DIR
                .get_file(path)
                .unwrap()
                .contents()
                .to_vec(),
        )
    }

    /// Return the sha3-256 hash of the compiled script bytes
    pub fn hash(self) -> HashValue {
        self.compiled_bytes().hash()
    }
}

/// Bytes produced by compiling a Move source language script into Move bytecode
#[derive(Clone)]
pub struct CompiledBytes(Vec<u8>);

impl CompiledBytes {
    /// Return the sha3-256 hash of the script bytes
    pub fn hash(&self) -> HashValue {
        Self::hash_bytes(&self.0)
    }

    /// Return the sha3-256 hash of the script bytes
    fn hash_bytes(bytes: &[u8]) -> HashValue {
        HashValue::from_sha3_256(bytes)
    }

    /// Convert this newtype wrapper into a vector of bytes
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl TryFrom<&[u8]> for StdlibScript {
    type Error = Error;

    /// Return `Some(<script_name>)` if  `code_bytes` is the bytecode of one of the standard library
    /// scripts, None otherwise.
    fn try_from(code_bytes: &[u8]) -> Result<Self> {
        let hash = CompiledBytes::hash_bytes(code_bytes);
        Self::all()
            .iter()
            .find(|script| script.hash() == hash)
            .cloned()
            .ok_or_else(|| anyhow!("Could not create standard library script from bytes"))
    }
}

impl fmt::Display for StdlibScript {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use StdlibScript::*;
        write!(
            f,
            "{}",
            match self {
                AddValidator => "add_validator",
                AddCurrency => "add_currency",
                AddCurrencyToAccount => "add_currency_to_account",
                AllowChildAccounts => "allow_child_accounts",
                ApplyForAssociationAddress => "apply_for_association_address",
                ApplyForAssociationPrivilege => "apply_for_association_privilege",
                ApplyForChildVaspCredential => "apply_for_child_vasp_credential",
                ApplyForParentCapability => "apply_for_parent_capability",
                ApplyForRootVasp => "apply_for_root_vasp",
                ApprovedPayment => "approved_payment",
                Burn => "burn",
                CancelBurn => "cancel_burn",
                CreateAccount => "create_account",
                CreateEmptyAccount => "create_empty_account",
                EmptyScript => "empty_script",
                GrantAssociationAddress => "grant_association_address",
                GrantAssociationPrivilege => "grant_association_privilege",
                GrantChildAccount => "grant_child_account",
                GrantParentAccount => "grant_parent_account",
                GrantVaspAccount => "grant_vasp_account",
                Mint => "mint",
                MintLbr => "mint_lbr",
                ModifyPublishingOption => "modify_publishing_option",
                PeerToPeer => "peer_to_peer",
                PeerToPeerWithMetadata => "peer_to_peer_with_metadata",
                Preburn => "preburn",
                PublishSharedEd2551PublicKey => "publish_shared_ed25519_public_key",
                RecertifyChildAccount => "recertify_child_account",
                RegisterApprovedPayment => "register_approved_payment",
                RegisterPreburner => "register_preburner",
                RegisterValidator => "register_validator",
                RemoveAssociationAddress => "remove_association_address",
                RemoveAssociationPrivilege => "remove_association_privilege",
                RemoveChildAccount => "remove_child_account",
                RemoveParentAccount => "remove_parent_account",
                RemoveValidator => "remove_validator",
                RotateAuthenticationKey => "rotate_authentication_key",
                RotateConsensusPubkey => "rotate_consensus_pubkey",
                RotateSharedEd2551PublicKey => "rotate_shared_ed25519_public_key",
                UnmintLbr => "unmint_lbr",
                UpdateLibraVersion => "update_libra_version",
                UpdateExchangeRate => "update_exchange_rate",
                UpdateMintingAbility => "update_minting_ability",
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_correspondence() {
        // make sure that every file under transaction_scripts/ is represented in
        // StdlibScript::all() (and vice versa)
        let files = STAGED_TXN_SCRIPTS_DIR.files();
        let scripts = StdlibScript::all();
        for file in files {
            assert!(
                StdlibScript::is(file.contents()),
                "File {} missing from StdlibScript enum",
                file.path().display()
            )
        }
        assert_eq!(
            files.len(),
            scripts.len(),
            "Mismatch between stdlib script files and StdlibScript enum. {}",
            if files.len() > scripts.len() {
                "Did you forget to extend the StdlibScript enum?"
            } else {
                "Did you forget to rebuild the standard library?"
            }
        );
    }
}
