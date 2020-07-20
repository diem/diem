// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants, error::Error, json_rpc::JsonRpcClientWrapper, secure_backend::SecureBackend,
    storage::StorageWrapper, TransactionContext,
};
use libra_crypto::{ed25519::Ed25519PublicKey, x25519, ValidCryptoMaterial};
use libra_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_KEY,
    VALIDATOR_NETWORK_KEY,
};
use libra_network_address::{
    encrypted::{EncNetworkAddress, Key, KeyVersion, RawEncNetworkAddress},
    NetworkAddress, RawNetworkAddress,
};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    account_address::{self, AccountAddress},
    chain_id::ChainId,
    transaction::{RawTransaction, Script, SignedTransaction},
    validator_config::ValidatorConfig,
};
use std::{convert::TryFrom, str::FromStr};

pub fn fetch_keys_from_storage(
    storage_name: &'static str,
    backend: &SecureBackend,
) -> Result<(Ed25519PublicKey, x25519::PublicKey, x25519::PublicKey), Error> {
    let storage = StorageWrapper::new(storage_name, &backend)?;
    let consensus_key = storage.ed25519_public_from_private(CONSENSUS_KEY)?;
    let fullnode_network_key = storage.x25519_public_from_private(FULLNODE_NETWORK_KEY)?;
    let validator_network_key = storage.x25519_public_from_private(VALIDATOR_NETWORK_KEY)?;
    Ok((consensus_key, fullnode_network_key, validator_network_key))
}

/// Encodes and encrypts a validator address
pub fn encode_validator_address(
    address: NetworkAddress,
    owner_account: &AccountAddress,
    sequence_number: u64,
    key: &Key,
    version: KeyVersion,
    addr_idx: u32,
) -> Result<RawEncNetworkAddress, Error> {
    let raw_address = encode_address(address)?;
    let enc_address =
        raw_address.encrypt(key, version, owner_account, sequence_number + 1, addr_idx);
    RawEncNetworkAddress::try_from(&enc_address).map_err(|e| {
        Error::UnexpectedError(format!(
            "error serializing encrypted address: '{:?}', error: {}",
            enc_address, e
        ))
    })
}

// Decodes adn decrypts the validator address
pub fn decode_validator_address(
    address: RawEncNetworkAddress,
    account: &AccountAddress,
    key: &Key,
    addr_idx: u32,
) -> Result<NetworkAddress, Error> {
    let enc_addr = EncNetworkAddress::try_from(&address).map_err(|e| {
        Error::UnexpectedError(format!(
            "Failed to decode network address {}",
            e.to_string()
        ))
    })?;
    let raw_addr = enc_addr.decrypt(key, account, addr_idx).map_err(|e| {
        Error::UnexpectedError(format!(
            "Failed to decrypt network address {}",
            e.to_string()
        ))
    })?;
    NetworkAddress::try_from(&raw_addr).map_err(|e| {
        Error::UnexpectedError(format!(
            "Failed to decode network address {}",
            e.to_string()
        ))
    })
}

/// Encode an address into bytes
pub fn encode_address(address: NetworkAddress) -> Result<RawNetworkAddress, Error> {
    RawNetworkAddress::try_from(&address).map_err(|e| {
        Error::UnexpectedError(format!(
            "error serializing address: '{}', error: {}",
            address, e
        ))
    })
}

pub fn create_validator_config_script(
    validator_account: AccountAddress,
    validator_config: ValidatorConfig,
) -> Script {
    // Generate the validator config script
    // TODO(philiphayes): remove network identity pubkey field from struct when
    // transition complete
    let consensus_key = validator_config.consensus_public_key;
    let validator_network_address = validator_config.validator_network_address;
    let validator_network_key = validator_config.validator_network_identity_public_key;
    let full_node_network_key = validator_config.full_node_network_identity_public_key;
    let full_node_network_address = validator_config.full_node_network_address;
    transaction_builder::encode_set_validator_config_script(
        validator_account,
        consensus_key.to_bytes().to_vec(),
        validator_network_key.to_bytes(),
        validator_network_address.into(),
        full_node_network_key.to_bytes(),
        full_node_network_address.into(),
    )
}

pub fn create_validator_config_and_reconfigure_script(
    validator_account: AccountAddress,
    validator_config: ValidatorConfig,
) -> Script {
    // Generate the validator config script
    // TODO(philiphayes): remove network identity pubkey field from struct when
    // transition complete
    let consensus_key = validator_config.consensus_public_key;
    let validator_network_address = validator_config.validator_network_address;
    let validator_network_key = validator_config.validator_network_identity_public_key;
    let full_node_network_key = validator_config.full_node_network_identity_public_key;
    let full_node_network_address = validator_config.full_node_network_address;
    transaction_builder::encode_set_validator_config_and_reconfigure_script(
        validator_account,
        consensus_key.to_bytes().to_vec(),
        validator_network_key.to_bytes(),
        validator_network_address.into(),
        full_node_network_key.to_bytes(),
        full_node_network_address.into(),
    )
}

pub fn create_transaction(
    storage_name: &'static str,
    backend: &SecureBackend,
    sequence_number: u64,
    chain_id: ChainId,
    script_name: &'static str,
    script: Script,
) -> Result<SignedTransaction, Error> {
    let mut storage = StorageWrapper::new(storage_name, &backend)?;
    let operator_address_string = storage.string(OPERATOR_ACCOUNT)?;
    let operator_address = AccountAddress::from_str(&operator_address_string)
        .map_err(|e| Error::BackendParsingError(e.to_string()))?;

    let expiration_time = RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS;
    let raw_transaction = RawTransaction::new_script(
        operator_address,
        sequence_number,
        script,
        constants::MAX_GAS_AMOUNT,
        constants::GAS_UNIT_PRICE,
        constants::GAS_CURRENCY_CODE.to_owned(),
        expiration_time,
        chain_id,
    );
    storage.sign(OPERATOR_KEY, script_name, raw_transaction)
}

/// Retrieves the owner key from the remote storage using the owner name given by
/// the validator-config command, and uses this key to derive an owner account address.
/// If a remote storage path is not specified, returns an error.
pub fn owner_from_key(
    storage_name: &'static str,
    backend: &SecureBackend,
    owner_name: String,
) -> Result<AccountAddress, Error> {
    let storage = StorageWrapper::new_with_namespace(storage_name, owner_name, backend)?;
    let owner_key = storage.ed25519_key(OWNER_KEY)?;
    Ok(account_address::from_public_key(&owner_key))
}

pub fn update_config_and_reconfigure(
    storage_name: &'static str,
    backend: &SecureBackend,
    client: &JsonRpcClientWrapper,
    owner_account: AccountAddress,
    chain_id: u8,
    sequence_number: u64,
    validator_config: ValidatorConfig,
) -> Result<TransactionContext, Error> {
    let validator_config_script =
        create_validator_config_and_reconfigure_script(owner_account, validator_config);

    // Create and sign the validator-config transaction
    let validator_config_tx = create_transaction(
        storage_name,
        backend,
        sequence_number,
        ChainId::new(chain_id),
        "validator-config",
        validator_config_script,
    )?;

    // Submit transaction to JSON-RPC
    client.submit_transaction(validator_config_tx)
}
