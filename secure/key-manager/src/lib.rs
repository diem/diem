// Copyright (c) The Diem Core Contributors
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
//! KeyManager talks to Diem via the DiemInterface that may either be a direct link into
//! `DiemDB`/`Executor`, JSON-RPC, or some other concoction.
//! KeyManager talks to its own storage through the `DiemSecureStorage::Storage trait.
#![forbid(unsafe_code)]

use crate::{
    counters::{
        KEYS_STILL_FRESH, LIVENESS_ERROR_ENCOUNTERED, ROTATED_IN_STORAGE,
        SUBMITTED_ROTATION_TRANSACTION, UNEXPECTED_ERROR_ENCOUNTERED, WAITING_ON_RECONFIGURATION,
        WAITING_ON_TRANSACTION_EXECUTION,
    },
    diem_interface::DiemInterface,
    logging::{LogEntry, LogEvent, LogSchema},
};
use diem_crypto::ed25519::Ed25519PublicKey;
use diem_global_constants::{CONSENSUS_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_ACCOUNT};
use diem_logger::prelude::*;
use diem_secure_storage::{CryptoStorage, KVStorage};
use diem_time_service::{TimeService, TimeServiceTrait};
use diem_types::{
    account_address::AccountAddress,
    account_config::XUS_NAME,
    chain_id::ChainId,
    transaction::{RawTransaction, SignedTransaction, Transaction},
};
use std::time::Duration;
use thiserror::Error;

pub mod counters;
pub mod diem_interface;
pub mod logging;

#[cfg(test)]
mod tests;

const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 400_000;

/// Defines actions that KeyManager should perform after a check of all associated state.
#[derive(Debug, PartialEq)]
pub enum Action {
    /// There is no need to perform a rotation (keys are still fresh).
    NoAction,
    /// Sufficient time has passed for another key rotation (keys are stale).
    FullKeyRotation,
    /// Storage and the blockchain are inconsistent, submit a new rotation transaction.
    SubmitKeyRotationTransaction,
    /// The validator config and the validator set are inconsistent, wait for reconfiguration.
    WaitForReconfiguration,
    /// Storage and the blockchain are inconsistent, wait for rotation transaction execution.
    WaitForTransactionExecution,
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
        "The diem_timestamp value on-chain isn't increasing. Last value: {0}, Current value: {1}"
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

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::UnknownError(format!("{}", error))
    }
}

impl From<diem_client::Error> for Error {
    fn from(error: diem_client::Error) -> Self {
        Error::UnknownError(format!("Client error: {}", error))
    }
}

impl From<bcs::Error> for Error {
    fn from(error: bcs::Error) -> Self {
        Error::UnknownError(format!("BCS error: {}", error))
    }
}

impl From<diem_secure_storage::Error> for Error {
    fn from(error: diem_secure_storage::Error) -> Self {
        Error::StorageError(error.to_string())
    }
}

pub struct KeyManager<LI, S> {
    diem: LI,
    storage: S,
    time_service: TimeService,
    last_checked_diem_timestamp: u64,
    rotation_period_secs: u64, // The frequency by which to rotate all keys
    sleep_period_secs: u64,    // The amount of time to sleep between key management checks
    txn_expiration_secs: u64,  // The time after which a rotation transaction expires
    chain_id: ChainId,
}

impl<LI, S> KeyManager<LI, S>
where
    LI: DiemInterface,
    S: KVStorage + CryptoStorage,
{
    pub fn new(
        diem: LI,
        storage: S,
        time_service: TimeService,
        rotation_period_secs: u64,
        sleep_period_secs: u64,
        txn_expiration_secs: u64,
        chain_id: ChainId,
    ) -> Self {
        Self {
            diem,
            storage,
            time_service,
            last_checked_diem_timestamp: 0,
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
        self.time_service
            .sleep_blocking(Duration::from_secs(self.sleep_period_secs));
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
        let validator_config = self.diem.retrieve_validator_config(owner_account)?;

        let storage_key = self.storage.get_public_key(CONSENSUS_KEY)?.public_key;
        let config_key = validator_config.consensus_public_key;
        if storage_key != config_key {
            return Err(Error::ConfigStorageKeyMismatch(config_key, storage_key));
        }

        Ok(())
    }

    pub fn compare_info_to_config(&self) -> Result<(), Error> {
        let owner_account = self.get_account_from_storage(OWNER_ACCOUNT)?;
        let validator_config = self.diem.retrieve_validator_config(owner_account)?;
        let validator_info = self.diem.retrieve_validator_info(owner_account)?;

        let info_key = validator_info.consensus_public_key();
        let config_key = validator_config.consensus_public_key;
        if &config_key != info_key {
            return Err(Error::ConfigInfoKeyMismatch(config_key, info_key.clone()));
        }

        Ok(())
    }

    pub fn last_reconfiguration(&self) -> Result<u64, Error> {
        // Convert the time to seconds
        Ok(self.diem.last_reconfiguration()? / 1_000_000)
    }

    pub fn last_rotation(&self) -> Result<u64, Error> {
        Ok(self.storage.get_public_key(CONSENSUS_KEY)?.last_update)
    }

    pub fn diem_timestamp(&self) -> Result<u64, Error> {
        // Convert the time to seconds
        Ok(self.diem.diem_timestamp()? / 1_000_000)
    }

    pub fn resubmit_consensus_key_transaction(&mut self) -> Result<(), Error> {
        let consensus_key = self.storage.get_public_key(CONSENSUS_KEY)?.public_key;
        self.submit_key_rotation_transaction(consensus_key)
            .map(|_| ())
    }

    pub fn rotate_consensus_key(&mut self) -> Result<Ed25519PublicKey, Error> {
        info!(LogSchema::new(LogEntry::KeyRotatedInStorage).event(LogEvent::Pending));
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
        info!(LogSchema::new(LogEntry::TransactionSubmitted).event(LogEvent::Pending));

        let operator_account = self.get_account_from_storage(OPERATOR_ACCOUNT)?;
        let seq_id = self.diem.retrieve_sequence_number(operator_account)?;
        let expiration = self.time_service.now_secs() + self.txn_expiration_secs;

        // Retrieve existing network information as registered on-chain
        let owner_account = self.get_account_from_storage(OWNER_ACCOUNT)?;
        let validator_config = self.diem.retrieve_validator_config(owner_account)?;

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

        self.diem
            .submit_transaction(Transaction::UserTransaction(signed_txn))?;

        info!(LogSchema::new(LogEntry::TransactionSubmitted).event(LogEvent::Success));
        counters::increment_metric_counter(SUBMITTED_ROTATION_TRANSACTION);

        Ok(consensus_key)
    }

    /// Ensures that the diem_timestamp() value registered on-chain is strictly monotonically
    /// increasing.
    fn ensure_timestamp_progress(&mut self) -> Result<(), Error> {
        let current_diem_timestamp = self.diem.diem_timestamp()?;
        if current_diem_timestamp <= self.last_checked_diem_timestamp {
            return Err(Error::LivenessError(
                self.last_checked_diem_timestamp,
                current_diem_timestamp,
            ));
        }

        self.last_checked_diem_timestamp = current_diem_timestamp;
        Ok(())
    }

    /// Evaluates the current status of the key manager by performing various state checks between
    /// secure storage and the blockchain.
    ///
    /// Note: every time this function is called, the diem_timestamp registered on-chain must be
    /// strictly monotonically increasing. This helps to ensure that the blockchain is making
    /// progress. Otherwise, if no progress is being made on-chain, a reconfiguration event is
    /// unlikely, and the key manager will be unable to rotate keys.
    pub fn evaluate_status(&mut self) -> Result<Action, Error> {
        self.ensure_timestamp_progress()?;

        // Compare the validator config to the validator set
        match self.compare_info_to_config() {
            Ok(()) => { /* Expected */ }
            Err(Error::ConfigInfoKeyMismatch(..)) => return Ok(Action::WaitForReconfiguration),
            Err(e) => return Err(e),
        }

        let last_rotation = self.last_rotation()?;

        // Compare the validator config to secure storage
        match self.compare_storage_to_config() {
            Ok(()) => { /* Expected */ }
            Err(Error::ConfigStorageKeyMismatch(..)) => {
                return if last_rotation + self.txn_expiration_secs <= self.time_service.now_secs() {
                    Ok(Action::SubmitKeyRotationTransaction)
                } else {
                    Ok(Action::WaitForTransactionExecution)
                };
            }
            Err(e) => return Err(e),
        };

        if last_rotation + self.rotation_period_secs <= self.time_service.now_secs() {
            Ok(Action::FullKeyRotation)
        } else {
            Ok(Action::NoAction)
        }
    }

    pub fn perform_action(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::FullKeyRotation => {
                info!(LogSchema::new(LogEntry::FullKeyRotation).event(LogEvent::Pending));
                self.rotate_consensus_key().map(|_| ())?;
                info!(LogSchema::new(LogEntry::FullKeyRotation).event(LogEvent::Success));
            }
            Action::SubmitKeyRotationTransaction => {
                info!(LogSchema::new(LogEntry::TransactionResubmission).event(LogEvent::Pending));
                self.resubmit_consensus_key_transaction()?;
                info!(LogSchema::new(LogEntry::TransactionResubmission).event(LogEvent::Success));
            }
            Action::NoAction => {
                info!(LogSchema::new(LogEntry::KeyStillFresh));
                counters::increment_metric_counter(KEYS_STILL_FRESH);
            }
            Action::WaitForReconfiguration => {
                warn!(LogSchema::new(LogEntry::WaitForReconfiguration));
                counters::increment_metric_counter(WAITING_ON_RECONFIGURATION);
            }
            Action::WaitForTransactionExecution => {
                warn!(LogSchema::new(LogEntry::WaitForTransactionExecution));
                counters::increment_metric_counter(WAITING_ON_TRANSACTION_EXECUTION);
            }
        };

        Ok(())
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
        diem_transaction_builder::stdlib::encode_set_validator_config_and_reconfigure_script(
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
        XUS_NAME.to_owned(),
        expiration_timestamp_secs,
        chain_id,
    )
}
