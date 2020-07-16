// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants,
    error::Error,
    secure_backend::{SharedBackend, ValidatorBackend},
};
use libra_config::config::HANDSHAKE_VERSION;
use libra_crypto::{ed25519::Ed25519PublicKey, x25519, ValidCryptoMaterial};
use libra_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_ACCOUNT, OWNER_KEY,
    VALIDATOR_NETWORK_KEY,
};
use libra_network_address::{
    encrypted::{
        RawEncNetworkAddress, TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION,
    },
    NetworkAddress, RawNetworkAddress,
};
use libra_secure_storage::{CryptoStorage, KVStorage, Storage, Value};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    account_address::{self, AccountAddress},
    chain_id::ChainId,
    transaction::{RawTransaction, Script, SignedTransaction, Transaction},
};
use std::{convert::TryFrom, str::FromStr};
use structopt::StructOpt;

// TODO(davidiw) add operator_address, since that will eventually be the identity producing this.
#[derive(Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long)]
    owner_name: String,
    #[structopt(long)]
    validator_address: NetworkAddress,
    #[structopt(long)]
    fullnode_address: NetworkAddress,
    #[structopt(long)]
    chain_id: ChainId,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    shared_backend: SharedBackend,
}

impl ValidatorConfig {
    pub fn execute(self) -> Result<Transaction, Error> {
        // Fetch the owner key from remote storage using the owner_name and derive an address
        let owner_account = self.fetch_owner_account()?;

        // Create the validator config script for the validator node
        let validator_config_script = self.create_validator_config_script(owner_account)?;

        // Create and sign the validator-config transaction
        let validator_config_tx =
            self.create_validator_config_transaction(validator_config_script)?;

        // Write validator config to local storage to save for verification later on
        let mut validator_storage = self
            .validator_backend
            .validator_backend
            .create_storage(self.validator_backend.name())?;

        validator_storage
            .set(
                constants::VALIDATOR_CONFIG,
                Value::Transaction(validator_config_tx.clone()),
            )
            .map_err(|e| {
                Error::StorageWriteError(
                    self.validator_backend.name(),
                    constants::VALIDATOR_CONFIG,
                    e.to_string(),
                )
            })?;

        // Save the owner account in local storage for deployment later on
        validator_storage
            .set(OWNER_ACCOUNT, Value::String(owner_account.to_string()))
            .map_err(|e| {
                Error::StorageWriteError(
                    self.validator_backend.name(),
                    OWNER_ACCOUNT,
                    e.to_string(),
                )
            })?;

        // Upload the validator config to shared storage
        let mut shared_storage = self
            .shared_backend
            .shared_backend
            .create_storage(self.shared_backend.name())?;
        shared_storage
            .set(
                constants::VALIDATOR_CONFIG,
                Value::Transaction(validator_config_tx.clone()),
            )
            .map_err(|e| {
                Error::StorageWriteError(
                    self.shared_backend.name(),
                    constants::VALIDATOR_CONFIG,
                    e.to_string(),
                )
            })?;

        Ok(validator_config_tx)
    }

    /// Creates and returns a validator config script using the keys stored in local storage. The
    /// validator address will be the given owner account address.
    fn create_validator_config_script(
        &self,
        owner_account: AccountAddress,
    ) -> Result<Script, Error> {
        // Retrieve keys from local storage
        let storage_name = self.validator_backend.name();
        let validator_storage = self
            .validator_backend
            .validator_backend
            .clone()
            .create_storage(storage_name)?;
        let consensus_key = ed25519_from_storage(CONSENSUS_KEY, &validator_storage, storage_name)?;
        let fullnode_network_key =
            x25519_from_storage(FULLNODE_NETWORK_KEY, &validator_storage, storage_name)?;
        let validator_network_key =
            x25519_from_storage(VALIDATOR_NETWORK_KEY, &validator_storage, storage_name)?;

        // Only supports one address for now
        let addr_idx = 0;

        // Append ln-noise-ik and ln-handshake protocols to base network addresses
        // and encrypt the validator address.
        let validator_address = self
            .validator_address
            .clone()
            .append_prod_protos(validator_network_key, HANDSHAKE_VERSION);
        let raw_validator_address =
            RawNetworkAddress::try_from(&validator_address).map_err(|e| {
                Error::UnexpectedError(format!(
                    "error serializing validator address: \"{}\", error: {}",
                    validator_address, e
                ))
            })?;
        // TODO(davidiw): In genesis this is irrelevant -- afterward we need to obtain the
        // current sequence number by querying the blockchain.
        let sequence_number = 0;
        let enc_validator_address = raw_validator_address.encrypt(
            &TEST_SHARED_VAL_NETADDR_KEY,
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            &owner_account,
            sequence_number,
            addr_idx,
        );
        let raw_enc_validator_address = RawEncNetworkAddress::try_from(&enc_validator_address)
            .map_err(|e| {
                Error::UnexpectedError(format!(
                    "error serializing encrypted validator address: {:?}, error: {}",
                    enc_validator_address, e
                ))
            })?;
        let fullnode_address = self
            .fullnode_address
            .clone()
            .append_prod_protos(fullnode_network_key, HANDSHAKE_VERSION);
        let raw_fullnode_address = RawNetworkAddress::try_from(&fullnode_address).map_err(|e| {
            Error::UnexpectedError(format!(
                "error serializing fullnode address: \"{}\", error: {}",
                fullnode_address, e
            ))
        })?;

        // Generate the validator config script
        // TODO(philiphayes): remove network identity pubkey field from struct when
        // transition complete
        Ok(transaction_builder::encode_set_validator_config_script(
            owner_account,
            consensus_key.to_bytes().to_vec(),
            validator_network_key.to_bytes(),
            raw_enc_validator_address.into(),
            fullnode_network_key.to_bytes(),
            raw_fullnode_address.into(),
        ))
    }

    /// Creates and returns a signed validator-config transaction.
    fn create_validator_config_transaction(&self, script: Script) -> Result<Transaction, Error> {
        let storage_name = self.validator_backend.name();
        let mut validator_storage = self
            .validator_backend
            .validator_backend
            .clone()
            .create_storage(storage_name)?;
        let operator_key = ed25519_from_storage(OPERATOR_KEY, &validator_storage, storage_name)?;
        let operator_address_string = validator_storage
            .get(OPERATOR_ACCOUNT)
            .and_then(|v| v.value.string())
            .map_err(|e| {
                Error::StorageReadError(
                    self.validator_backend.name(),
                    OPERATOR_ACCOUNT,
                    e.to_string(),
                )
            })?;
        let operator_address = AccountAddress::from_str(&operator_address_string)
            .map_err(|e| Error::BackendParsingError(e.to_string()))?;

        // TODO(joshlind): In genesis the sequence number is irrelevant. After genesis we need to
        // obtain the current sequence number by querying the blockchain.
        let sequence_number = 0;
        let expiration_time = RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS;
        let raw_transaction = RawTransaction::new_script(
            operator_address,
            sequence_number,
            script,
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            expiration_time,
            self.chain_id,
        );

        let signature = validator_storage
            .sign(OPERATOR_KEY, &raw_transaction)
            .map_err(|e| {
                Error::StorageSigningError(
                    self.validator_backend.name(),
                    "validator-config",
                    OPERATOR_KEY,
                    e.to_string(),
                )
            })?;
        let signed_txn = SignedTransaction::new(raw_transaction, operator_key, signature);
        Ok(Transaction::UserTransaction(signed_txn))
    }

    /// Retrieves the owner key from the remote storage using the owner name given by
    /// the validator-config command, and uses this key to derive an owner account address.
    /// If a remote storage path is not specified, returns an error.
    fn fetch_owner_account(&self) -> Result<AccountAddress, Error> {
        let owner_storage = self
            .shared_backend
            .shared_backend
            .clone()
            .set_namespace(self.owner_name.clone())
            .create_storage(self.shared_backend.name())?;
        let owner_key = owner_storage
            .get(OWNER_KEY)
            .map_err(|e| {
                Error::StorageReadError(self.shared_backend.name(), OWNER_KEY, e.to_string())
            })?
            .value
            .ed25519_public_key()
            .map_err(|e| {
                Error::StorageReadError(self.shared_backend.name(), OWNER_KEY, e.to_string())
            })?;
        Ok(account_address::from_public_key(&owner_key))
    }
}

fn ed25519_from_storage(
    key_name: &'static str,
    storage: &Storage,
    storage_name: &'static str,
) -> Result<Ed25519PublicKey, Error> {
    Ok(storage
        .get_public_key(key_name)
        .map_err(|e| Error::StorageReadError(storage_name, key_name, e.to_string()))?
        .public_key)
}

fn x25519_from_storage(
    key_name: &'static str,
    storage: &Storage,
    storage_name: &'static str,
) -> Result<x25519::PublicKey, Error> {
    let edkey = ed25519_from_storage(key_name, storage, storage_name)?;
    x25519::PublicKey::from_ed25519_public_bytes(&edkey.to_bytes())
        .map_err(|e| Error::UnexpectedError(e.to_string()))
}
