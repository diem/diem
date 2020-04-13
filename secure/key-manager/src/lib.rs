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
//! KeyManager talks to its own storage through the `LibraSecureStorage::Storage trait.
#![forbid(unsafe_code)]

use libra_crypto::{ed25519, TPrivateKey};
use libra_secure_storage::Storage;
use libra_secure_time::TimeService;
use libra_transaction_scripts;
use libra_types::{
    account_address::AccountAddress,
    account_config::LBR_NAME,
    transaction::{RawTransaction, Script, Transaction, TransactionArgument},
    validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
};
use std::time::Duration;
use thiserror::Error;

#[cfg(test)]
mod tests;

pub const ACCOUNT_KEY: &str = "account_key";
pub const CONSENSUS_KEY: &str = "consensus_key";
const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 400_000;
const ROTATION_PERIOD_SECS: u64 = 604_800; // 1 week
const TXN_EXPIRATION_SECS: u64 = 3600; // 1 hour, we'll try again after that
const TXN_RETRY_SECS: u64 = 3600; // 1 hour retry period

/// Defines actions that KeyManager should perform after a check of all associated state.
#[derive(Debug, PartialEq)]
pub enum Action {
    /// The system is in a healthy state and there is no need to perform a rotation
    NoAction,
    /// The system is in a healthy state but sufficient time has passed for another key rotation
    FullKeyRotation,
    /// Storage and the blockchain are inconsistent, submit a new rotation
    SubmitKeyRotationTransaction,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Unknown error: {0}")]
    UnknownError(String),
    #[error("Key mismatch, config: {0}, info: {0}")]
    ConfigInfoKeyMismatch(ed25519::VerifyingKey, ed25519::VerifyingKey),
    #[error("Key mismatch, config: {0}, storage: {0}")]
    ConfigStorageKeyMismatch(ed25519::VerifyingKey, ed25519::VerifyingKey),
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
pub trait LibraInterface {
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

pub struct KeyManager<LI, S, T> {
    account: AccountAddress,
    key_name: String,
    libra: LI,
    storage: S,
    time_service: T,
}

impl<LI, S, T> KeyManager<LI, S, T>
where
    LI: LibraInterface,
    S: Storage,
    T: TimeService,
{
    pub fn new(
        account: AccountAddress,
        key_name: String,
        libra: LI,
        storage: S,
        time_service: T,
    ) -> Self {
        Self {
            account,
            key_name,
            libra,
            storage,
            time_service,
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

    pub fn resubmit_consensus_key_transaction(&self) -> Result<(), Error> {
        let storage_key = self.storage.get_public_key(&self.key_name)?.public_key;
        self.submit_key_rotation_transaction(storage_key)
            .map(|_| ())
    }

    pub fn rotate_consensus_key(&mut self) -> Result<ed25519::VerifyingKey, Error> {
        let new_key = self.storage.rotate_key(CONSENSUS_KEY)?;
        self.submit_key_rotation_transaction(new_key)
    }

    pub fn submit_key_rotation_transaction(
        &self,
        new_key: ed25519::VerifyingKey,
    ) -> Result<ed25519::VerifyingKey, Error> {
        let account_prikey = self.storage.export_private_key(ACCOUNT_KEY)?;
        let seq_id = self.libra.retrieve_sequence_number(self.account)?;
        let expiration = Duration::from_secs(self.time_service.now() + TXN_EXPIRATION_SECS);
        let txn =
            build_rotation_transaction(self.account, seq_id, &account_prikey, &new_key, expiration);
        self.libra.submit_transaction(txn)?;
        Ok(new_key)
    }

    pub fn evaluate_status(&self) -> Result<Action, Error> {
        // If this is inconsistent, then we are likely waiting on a reconfiguration.
        if let Err(Error::ConfigInfoKeyMismatch(..)) = self.compare_info_to_config() {
            // For now, just assume this is correct, but this needs to compare libra_timestamp with
            // the current time to ensure that progress is being made. And if not flag an error.
            return Ok(Action::NoAction);
        }

        let last_rotation = self.last_rotation()?;

        // If this is inconsistent, then the transaction either failed or was never submitted.
        if let Err(Error::ConfigStorageKeyMismatch(..)) = self.compare_storage_to_config() {
            return if last_rotation + TXN_RETRY_SECS <= self.time_service.now() {
                Ok(Action::SubmitKeyRotationTransaction)
            } else {
                Ok(Action::NoAction)
            };
        }

        if last_rotation + ROTATION_PERIOD_SECS <= self.time_service.now() {
            Ok(Action::FullKeyRotation)
        } else {
            Ok(Action::NoAction)
        }
    }

    pub fn perform_action(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::FullKeyRotation => self.rotate_consensus_key().map(|_| ()),
            Action::SubmitKeyRotationTransaction => self.resubmit_consensus_key_transaction(),
            Action::NoAction => Ok(()),
        }
    }
}

pub fn build_rotation_transaction(
    sender: AccountAddress,
    seq_id: u64,
    signing_key: &ed25519::SigningKey,
    new_key: &ed25519::VerifyingKey,
    expiration: Duration,
) -> Transaction {
    let script = Script::new(
        libra_transaction_scripts::ROTATE_CONSENSUS_PUBKEY_TXN.clone(),
        vec![],
        vec![TransactionArgument::U8Vector(new_key.to_bytes().to_vec())],
    );
    let raw_txn = RawTransaction::new_script(
        sender,
        seq_id,
        script,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        LBR_NAME.to_string(),
        expiration,
    );
    let signed_txn = raw_txn.sign(signing_key, signing_key.public_key()).unwrap();
    Transaction::UserTransaction(signed_txn.into_inner())
}
