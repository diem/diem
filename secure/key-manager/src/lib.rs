// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The purpose of KeyManager is to rotate consensus key and eventually the network key. It is not
//! responsible for generating the first key and fails if the stores have not been properly setup.
//! During rotation, it first updates the local store, then submits a transaction to rotate to the
//! new key. After some period of time and upon restarts of the process, it will evaluate the
//! current status of the system including:
//! * last rotation time, and rotate if it is too long ago
//! * if the latest key in the store matches the latest key in the ValidatorConfig, upon mismatch
//! it will try to submit a transaction to update the ValidatorConfig to the current key in the
//! store.
//! * if the current key in the ValidatorConfig matches the ValidatorSet, if it does not it
//! evaluates the current time from the last reconfiguration and logs that delta with greater
//! levels of severity depending on the delta.
//!
//! KeyManager talks to Libra via the LibraInterface that may either be a direct link into
//! `LibraDB`/`Executor`, JSON-RPC, or some other concoction.
//! KeyManager talks to its own storage through the `LibraSecureStorage::Storage1 trait.
#![forbid(unsafe_code)]

use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey,
};
use libra_transaction_scripts;
use libra_types::{
    account_address::AccountAddress,
    account_config::lbr_type_tag,
    transaction::{RawTransaction, Script, Transaction, TransactionArgument},
};
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[cfg(test)]
use libra_secure_storage::Storage;
#[cfg(test)]
use libra_types::{validator_config::ValidatorConfig, validator_info::ValidatorInfo};

#[cfg(test)]
mod tests;

pub const ACCOUNT_KEY: &str = "account_key";
pub const CONSENSUS_KEY: &str = "consensus_key";
const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 400_000;
const TXN_EXPIRATION: u64 = 100;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Unknown error: {0}")]
    UnknownError(String),
    #[error("Key mismatch, config: {0}, info: {0}")]
    ConfigInfoKeyMismatch(Ed25519PublicKey, Ed25519PublicKey),
    #[error("Key mismatch, config: {0}, storage: {0}")]
    ConfigStorageKeyMismatch(Ed25519PublicKey, Ed25519PublicKey),
    #[error("Data does not exist: {0}")]
    DataDoesNotExist(&'static str),
    #[error("Internal storage error")]
    SecureStorageError(#[from] libra_secure_storage::Error),
    #[error("ValidatorInfo not found in ValidatorConfig: {0}")]
    ValidatorInfoNotFound(AccountAddress),
}

#[cfg(test)]
impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::UnknownError(format!("{}", error))
    }
}

/// This defines a generic trait used to interact with the Libra blockchain. In production, this
/// will be talking to a JSON-RPC service. For tests, this may be an executor and storage directly.
#[cfg(test)]
trait LibraInterface {
    /// Retrieves the current time from the blockchain, this is returned as microseconds.
    fn libra_timestamp(&self) -> Result<u64, Error>;

    /// Retrieves the last reconfiguration time from the blockchain, this is returned as
    /// microseconds.
    fn last_reconfiguration(&self) -> Result<u64, Error>;

    /// Retrieve current sequence number for the provided account.
    fn retrieve_sequence_number(&self, account: AccountAddress) -> Result<u64, Error>;

    /// Submits a transaction to the block chain and returns successfully if the transaction was
    /// successfully submitted. It does not necessarily mean the transaction successfully executed.
    fn submit_transaction(&self, transaction: Transaction) -> Result<(), Error>;

    /// Retrieves the ValidatorConfig at the specified AccountAddress if one exists.
    fn retrieve_validator_config(&self, account: AccountAddress) -> Result<ValidatorConfig, Error>;

    /// Retrieves the ValidatorInfo for the specified account from the current ValidatorSet if one exists.
    fn retrieve_validator_info(&self, account: AccountAddress) -> Result<ValidatorInfo, Error>;
}

#[cfg(test)]
struct KeyManager<LI, S> {
    account: AccountAddress,
    key_name: String,
    libra: LI,
    storage: S,
}

#[cfg(test)]
impl<LI, S> KeyManager<LI, S>
where
    LI: LibraInterface,
    S: Storage,
{
    pub fn new(account: AccountAddress, key_name: String, libra: LI, storage: S) -> Self {
        Self {
            account,
            key_name,
            libra,
            storage,
        }
    }

    pub fn compare_storage_to_config(&self) -> Result<(), Error> {
        let storage_key = self.storage.get_public_key(&self.key_name)?.public_key;
        let validator_config = self.libra.retrieve_validator_config(self.account)?;
        let config_key = validator_config.consensus_pubkey;

        if storage_key == config_key {
            return Ok(());
        }
        Err(Error::ConfigStorageKeyMismatch(config_key, storage_key))
    }

    pub fn compare_info_to_config(&self) -> Result<(), Error> {
        let validator_info = self.libra.retrieve_validator_info(self.account)?;
        let info_key = validator_info.consensus_public_key();
        let validator_config = self.libra.retrieve_validator_config(self.account)?;
        let config_key = validator_config.consensus_pubkey;

        if &config_key == info_key {
            return Ok(());
        }
        Err(Error::ConfigInfoKeyMismatch(config_key, info_key.clone()))
    }

    pub fn last_reconfiguration(&self) -> Result<u64, Error> {
        // Convert the time to seconds
        Ok(self.libra.last_reconfiguration()? / 1_000_000)
    }

    pub fn last_rotation(&self) -> Result<u64, Error> {
        Ok(self.storage.get_public_key(&self.key_name)?.last_update)
    }

    pub fn libra_timestamp(&self) -> Result<u64, Error> {
        // Convert the time to seconds
        Ok(self.libra.libra_timestamp()? / 1_000_000)
    }

    pub fn rotate_consensus_key(&mut self) -> Result<Ed25519PublicKey, Error> {
        let new_key = self.storage.rotate_key(CONSENSUS_KEY)?;
        let account_prikey = self.storage.get_private_key(ACCOUNT_KEY)?;
        let seq_id = self.libra.retrieve_sequence_number(self.account)?;
        let transaction = build_transaction(self.account, seq_id, &account_prikey, &new_key);
        self.libra.submit_transaction(transaction)?;
        Ok(new_key)
    }
}

fn expiration_time(until: u64) -> Duration {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    std::time::Duration::from_secs(now + until)
}

pub fn build_transaction(
    sender: AccountAddress,
    seq_id: u64,
    signing_key: &Ed25519PrivateKey,
    new_key: &Ed25519PublicKey,
) -> Transaction {
    let script = Script::new(
        libra_transaction_scripts::ROTATE_CONSENSUS_PUBKEY_TXN.clone(),
        vec![TransactionArgument::U8Vector(new_key.to_bytes().to_vec())],
    );
    let raw_txn = RawTransaction::new_script(
        sender,
        seq_id,
        script,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        lbr_type_tag(),
        expiration_time(TXN_EXPIRATION),
    );
    let signed_txn = raw_txn.sign(signing_key, signing_key.public_key()).unwrap();
    Transaction::UserTransaction(signed_txn.into_inner())
}
