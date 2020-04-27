// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The purpose of KeyManager is to rotate consensus key (and eventually the network key). It is not
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

use crate::libra_interface::LibraInterface;
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey,
};
use libra_secure_storage::Storage;
use libra_secure_time::TimeService;
use libra_types::{
    account_address::AccountAddress,
    transaction::{RawTransaction, Script, Transaction, TransactionArgument},
};
use std::time::Duration;
use thiserror::Error;

pub mod libra_interface;

#[cfg(test)]
mod tests;

pub const VALIDATOR_KEY: &str = "validator";
pub const CONSENSUS_KEY: &str = "consensus";
const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 400_000;
const ROTATION_PERIOD_SECS: u64 = 604_800; // 1 week
const SLEEP_PERIOD_SECS: u64 = 600; // 10 minutes, after which the key manager will awaken again
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
    ConfigInfoKeyMismatch(Ed25519PublicKey, Ed25519PublicKey),
    #[error("Key mismatch, config: {0}, storage: {0}")]
    ConfigStorageKeyMismatch(Ed25519PublicKey, Ed25519PublicKey),
    #[error("Data does not exist: {0}")]
    DataDoesNotExist(String),
    #[error(
        "The libra_timestamp value on-chain isn't increasing. Last value: {0}, Current value: {0}:"
    )]
    LivenessError(u64, u64),
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

pub struct KeyManager<LI, S, T> {
    account: AccountAddress,
    libra: LI,
    storage: S,
    time_service: T,
    last_checked_libra_timestamp: u64,
}

impl<LI, S, T> KeyManager<LI, S, T>
where
    LI: LibraInterface,
    S: Storage,
    T: TimeService,
{
    pub fn new(account: AccountAddress, libra: LI, storage: S, time_service: T) -> Self {
        Self {
            account,
            libra,
            storage,
            time_service,
            last_checked_libra_timestamp: 0,
        }
    }

    /// Begins execution of the key manager by running an infinite loop where the key manager will
    /// periodically wake up, verify the state of the validator keys (e.g., the consensus key), and
    /// initiate a key rotation when required. If something goes wrong, an error will be returned
    /// by this method, upon which the key manager will flag the error and stop execution.
    pub fn execute(&mut self) -> Result<(), Error> {
        loop {
            self.execute_once()?;
            self.time_service.sleep(SLEEP_PERIOD_SECS);
        }
    }

    /// Checks the current state of the validator keys and performs any actions that might be
    /// required (e.g., performing a key rotation).
    pub fn execute_once(&mut self) -> Result<(), Error> {
        let status = self.evaluate_status()?;
        if status != Action::NoAction {
            self.perform_action(status)?;
        }
        Ok(())
    }

    pub fn compare_storage_to_config(&self) -> Result<(), Error> {
        let storage_key = self.storage.get_public_key(CONSENSUS_KEY)?.public_key;
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
        Ok(self.storage.get_public_key(CONSENSUS_KEY)?.last_update)
    }

    pub fn libra_timestamp(&self) -> Result<u64, Error> {
        // Convert the time to seconds
        Ok(self.libra.libra_timestamp()? / 1_000_000)
    }

    pub fn resubmit_consensus_key_transaction(&self) -> Result<(), Error> {
        let storage_key = self.storage.get_public_key(CONSENSUS_KEY)?.public_key;
        self.submit_key_rotation_transaction(storage_key)
            .map(|_| ())
    }

    pub fn rotate_consensus_key(&mut self) -> Result<Ed25519PublicKey, Error> {
        let new_key = self.storage.rotate_key(CONSENSUS_KEY)?;
        self.submit_key_rotation_transaction(new_key)
    }

    pub fn submit_key_rotation_transaction(
        &self,
        new_key: Ed25519PublicKey,
    ) -> Result<Ed25519PublicKey, Error> {
        let account_prikey = self.storage.export_private_key(VALIDATOR_KEY)?;
        let seq_id = self.libra.retrieve_sequence_number(self.account)?;
        let expiration = Duration::from_secs(self.time_service.now() + TXN_EXPIRATION_SECS);
        let txn =
            build_rotation_transaction(self.account, seq_id, &account_prikey, &new_key, expiration);
        self.libra.submit_transaction(txn)?;
        Ok(new_key)
    }

    /// Ensures that the libra_timestamp() value registered on-chain is strictly monotonically
    /// increasing.
    fn ensure_timestamp_progress(&mut self) -> Result<(), Error> {
        let current_libra_timestamp = self.libra.libra_timestamp()?;
        if current_libra_timestamp <= self.last_checked_libra_timestamp {
            return Err(Error::LivenessError(
                self.last_checked_libra_timestamp,
                current_libra_timestamp,
            ));
        }

        self.last_checked_libra_timestamp = current_libra_timestamp;
        Ok(())
    }

    /// Evaluates the current status of the key manager by performing various state checks between
    /// secure storage and the blockchain.
    ///
    /// Note: every time this function is called, the libra_timestamp registered on-chain must be
    /// strictly monotonically increasing. This helps to ensure that the blockchain is making
    /// progress. Otherwise, if no progress is being made on-chain, a reconfiguration event is
    /// unlikely, and the key manager will be unable to rotate keys.
    pub fn evaluate_status(&mut self) -> Result<Action, Error> {
        self.ensure_timestamp_progress()?;

        // If this is inconsistent, then we are waiting on a reconfiguration...
        if let Err(Error::ConfigInfoKeyMismatch(..)) = self.compare_info_to_config() {
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
    signing_key: &Ed25519PrivateKey,
    new_key: &Ed25519PublicKey,
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
        expiration,
    );
    let signed_txn = raw_txn.sign(signing_key, signing_key.public_key()).unwrap();
    Transaction::UserTransaction(signed_txn.into_inner())
}
