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

use crate::{
    counters::{
        KEYS_STILL_FRESH, LIVENESS_ERROR_ENCOUNTERED, NO_ACTION, ROTATED_IN_STORAGE,
        SUBMITTED_ROTATION_TRANSACTION, UNEXPECTED_ERROR_ENCOUNTERED, WAITING_ON_RECONFIGURATION,
        WAITING_ON_TRANSACTION_EXECUTION,
    },
    libra_interface::LibraInterface,
    logging::{LogEntry, LogEvent, LogSchema},
};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_global_constants::{CONSENSUS_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_ACCOUNT};
use libra_logger::prelude::*;
use libra_secure_storage::{CryptoStorage, KVStorage};
use libra_secure_time::TimeService;
use libra_types::{
    account_address::AccountAddress,
    account_config::LBR_NAME,
    chain_id::ChainId,
    transaction::{RawTransaction, SignedTransaction, Transaction},
};
use thiserror::Error;

pub mod counters;
pub mod libra_interface;
pub mod logging;

#[cfg(test)]
mod tests;

const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 400_000;

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
    #[error("Key mismatch, config: {0}, info: {1}")]
    ConfigInfoKeyMismatch(Ed25519PublicKey, Ed25519PublicKey),
    #[error("Key mismatch, config: {0}, storage: {1}")]
    ConfigStorageKeyMismatch(Ed25519PublicKey, Ed25519PublicKey),
    #[error("Data does not exist: {0}")]
    DataDoesNotExist(String),
    #[error(
        "The libra_timestamp value on-chain isn't increasing. Last value: {0}, Current value: {1}"
    )]
    LivenessError(u64, u64),
    #[error("Unable to retrieve the account address: {0}, storage error: {1}")]
    MissingAccountAddress(String, String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("ValidatorInfo not found in ValidatorConfig: {0}")]
    ValidatorInfoNotFound(AccountAddress),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

#[cfg(test)]
impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::UnknownError(format!("{}", error))
    }
}

impl From<libra_secure_storage::Error> for Error {
    fn from(error: libra_secure_storage::Error) -> Self {
        Error::StorageError(error.to_string())
    }
}

pub struct KeyManager<LI, S, T> {
    libra: LI,
    storage: S,
    time_service: T,
    last_checked_libra_timestamp: u64,
    rotation_period_secs: u64, // The frequency by which to rotate all keys
    sleep_period_secs: u64,    // The amount of time to sleep between key management checks
    txn_expiration_secs: u64,  // The time after which a rotation transaction expires
    chain_id: ChainId,
}

impl<LI, S, T> KeyManager<LI, S, T>
where
    LI: LibraInterface,
    S: KVStorage + CryptoStorage,
    T: TimeService,
{
    pub fn new(
        libra: LI,
        storage: S,
        time_service: T,
        rotation_period_secs: u64,
        sleep_period_secs: u64,
        txn_expiration_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        Self {
            libra,
            storage,
            time_service,
            last_checked_libra_timestamp: 0,
            rotation_period_secs,
            sleep_period_secs,
            txn_expiration_secs,
            chain_id,
        }
    }

    /// Begins execution of the key manager by running an infinite loop where the key manager will
    /// periodically wake up, verify the state of the validator keys (e.g., the consensus key), and
    /// initiate a key rotation when required. If something goes wrong that we can't handle, an
    /// error will be returned by this method, upon which the key manager will flag the error and
    /// stop execution.
    pub fn execute(&mut self) -> Result<(), Error> {
        loop {
            info!(LogSchema::new(LogEntry::CheckKeyStatus).event(LogEvent::Pending));

            match self.execute_once() {
                Ok(_) => {
                    info!(LogSchema::new(LogEntry::CheckKeyStatus).event(LogEvent::Success));
                }
                Err(Error::LivenessError(last_value, current_value)) => {
                    // Log the liveness error and continue to execute.
                    let error = Error::LivenessError(last_value, current_value);
                    error!(LogSchema::new(LogEntry::CheckKeyStatus)
                        .event(LogEvent::Error)
                        .liveness_error(&error));
                    counters::increment_metric_counter(LIVENESS_ERROR_ENCOUNTERED);
                }
                Err(e) => {
                    // Log the unexpected error and continue to execute.
                    error!(LogSchema::new(LogEntry::CheckKeyStatus)
                        .event(LogEvent::Error)
                        .unexpected_error(&e));
                    counters::increment_metric_counter(UNEXPECTED_ERROR_ENCOUNTERED);
                }
            };

            self.sleep();
        }
    }

    fn sleep(&self) {
        info!(LogSchema::new(LogEntry::Sleep)
            .event(LogEvent::Pending)
            .sleep_duration(self.sleep_period_secs));
        self.time_service.sleep(self.sleep_period_secs);

        info!(LogSchema::new(LogEntry::Sleep).event(LogEvent::Success));
    }

    /// Checks the current state of the validator keys and performs any actions that might be
    /// required (e.g., performing a key rotation).
    pub fn execute_once(&mut self) -> Result<(), Error> {
        let action = self.evaluate_status()?;
        self.perform_action(action)
    }

    pub fn compare_storage_to_config(&self) -> Result<(), Error> {
        let owner_account = self.get_account_from_storage(OWNER_ACCOUNT)?;
        let validator_config = self.libra.retrieve_validator_config(owner_account)?;

        let storage_key = self.storage.get_public_key(CONSENSUS_KEY)?.public_key;
        let config_key = validator_config.consensus_public_key;
        if storage_key != config_key {
            return Err(Error::ConfigStorageKeyMismatch(config_key, storage_key));
        }

        Ok(())
    }

    pub fn compare_info_to_config(&self) -> Result<(), Error> {
        let owner_account = self.get_account_from_storage(OWNER_ACCOUNT)?;
        let validator_config = self.libra.retrieve_validator_config(owner_account)?;
        let validator_info = self.libra.retrieve_validator_info(owner_account)?;

        let info_key = validator_info.consensus_public_key();
        let config_key = validator_config.consensus_public_key;
        if &config_key != info_key {
            return Err(Error::ConfigInfoKeyMismatch(config_key, info_key.clone()));
        }

        Ok(())
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

    pub fn resubmit_consensus_key_transaction(&mut self) -> Result<(), Error> {
        let consensus_key = self.storage.get_public_key(CONSENSUS_KEY)?.public_key;
        counters::increment_metric_counter(SUBMITTED_ROTATION_TRANSACTION);
        self.submit_key_rotation_transaction(consensus_key)
            .map(|_| ())
    }

    pub fn rotate_consensus_key(&mut self) -> Result<Ed25519PublicKey, Error> {
        let consensus_key = self.storage.rotate_key(CONSENSUS_KEY)?;
        info!(LogSchema::new(LogEntry::KeyRotatedInStorage)
            .event(LogEvent::Success)
            .consensus_key(&consensus_key));
        counters::increment_metric_counter(ROTATED_IN_STORAGE);
        self.submit_key_rotation_transaction(consensus_key)
    }

    pub fn submit_key_rotation_transaction(
        &mut self,
        consensus_key: Ed25519PublicKey,
    ) -> Result<Ed25519PublicKey, Error> {
        let operator_account = self.get_account_from_storage(OPERATOR_ACCOUNT)?;
        let seq_id = self.libra.retrieve_sequence_number(operator_account)?;
        let expiration = self.time_service.now() + self.txn_expiration_secs;

        // Retrieve existing network information as registered on-chain
        let owner_account = self.get_account_from_storage(OWNER_ACCOUNT)?;
        let validator_config = self.libra.retrieve_validator_config(owner_account)?;

        let txn = build_rotation_transaction(
            owner_account,
            operator_account,
            seq_id,
            &consensus_key,
            validator_config.validator_network_addresses,
            validator_config.fullnode_network_addresses,
            expiration,
            self.chain_id,
        );

        let operator_pubkey = self.storage.get_public_key(OPERATOR_KEY)?.public_key;
        let txn_signature = self.storage.sign(OPERATOR_KEY, &txn)?;
        let signed_txn = SignedTransaction::new(txn, operator_pubkey, txn_signature);

        self.libra
            .submit_transaction(Transaction::UserTransaction(signed_txn))?;
        info!(LogSchema::new(LogEntry::TransactionSubmission).event(LogEvent::Success));

        Ok(consensus_key)
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
            warn!(LogSchema::new(LogEntry::WaitForReconfiguration));
            counters::increment_metric_counter(WAITING_ON_RECONFIGURATION);
            return Ok(Action::NoAction);
        }

        let last_rotation = self.last_rotation()?;

        // If this is inconsistent, then the transaction either failed or was never submitted.
        if let Err(Error::ConfigStorageKeyMismatch(..)) = self.compare_storage_to_config() {
            return if last_rotation + self.txn_expiration_secs <= self.time_service.now() {
                Ok(Action::SubmitKeyRotationTransaction)
            } else {
                warn!(LogSchema::new(LogEntry::WaitForTransactionExecution));
                counters::increment_metric_counter(WAITING_ON_TRANSACTION_EXECUTION);
                Ok(Action::NoAction)
            };
        }

        if last_rotation + self.rotation_period_secs <= self.time_service.now() {
            Ok(Action::FullKeyRotation)
        } else {
            info!(LogSchema::new(LogEntry::KeyStillFresh));
            counters::increment_metric_counter(KEYS_STILL_FRESH);
            Ok(Action::NoAction)
        }
    }

    pub fn perform_action(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::FullKeyRotation => {
                info!(LogSchema::new(LogEntry::FullKeyRotation).event(LogEvent::Pending));
                self.rotate_consensus_key().map(|_| ())
            }
            Action::SubmitKeyRotationTransaction => {
                info!(LogSchema::new(LogEntry::TransactionSubmission).event(LogEvent::Pending));
                self.resubmit_consensus_key_transaction()
            }
            Action::NoAction => {
                info!(LogSchema::new(LogEntry::NoAction));
                counters::increment_metric_counter(NO_ACTION);
                Ok(())
            }
        }
    }

    fn get_account_from_storage(&self, account_name: &str) -> Result<AccountAddress, Error> {
        self.storage
            .get::<AccountAddress>(account_name)
            .map(|v| v.value)
            .map_err(|e| Error::MissingAccountAddress(account_name.into(), e.to_string()))
    }
}

pub fn build_rotation_transaction(
    owner_address: AccountAddress,
    operator_address: AccountAddress,
    seq_id: u64,
    consensus_key: &Ed25519PublicKey,
    network_addresses: Vec<u8>,
    fullnode_network_addresses: Vec<u8>,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawTransaction {
    let script =
        transaction_builder_generated::stdlib::encode_set_validator_config_and_reconfigure_script(
            owner_address,
            consensus_key.to_bytes().to_vec(),
            network_addresses,
            fullnode_network_addresses,
        );
    RawTransaction::new_script(
        operator_address,
        seq_id,
        script,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        LBR_NAME.to_owned(),
        expiration_timestamp_secs,
        chain_id,
    )
}
