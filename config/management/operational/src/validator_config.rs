// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, TransactionContext};
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, OPERATOR_ACCOUNT, OWNER_ACCOUNT, VALIDATOR_NETWORK_KEY,
};
use libra_management::{error::Error, storage::to_x25519};
use libra_network_address::{
    encrypted::{EncNetworkAddress, Key, RawEncNetworkAddress, TEST_SHARED_VAL_NETADDR_KEY},
    NetworkAddress, Protocol, RawNetworkAddress,
};
use libra_types::account_address::AccountAddress;
use serde::Serialize;
use std::convert::TryFrom;
use structopt::StructOpt;

// TODO: Load all chain IDs from the host
#[derive(Debug, StructOpt)]
pub struct SetValidatorConfig {
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
    #[structopt(long, help = "Validator Network Address")]
    validator_address: Option<NetworkAddress>,
    #[structopt(long, help = "Full Node Network Address")]
    fullnode_address: Option<NetworkAddress>,
}

impl SetValidatorConfig {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let config = self
            .validator_config
            .config
            .load()?
            .override_json_server(&self.json_server)
            .override_validator_backend(
                &self.validator_config.validator_backend.validator_backend,
            )?;
        let storage = config.validator_backend();

        let operator_account = storage.account_address(OPERATOR_ACCOUNT)?;
        let owner_account = storage.account_address(OWNER_ACCOUNT)?;
        let client = JsonRpcClientWrapper::new(config.json_server);
        let sequence_number = client.sequence_number(operator_account)?;

        // Retrieve the current validator / fullnode addresses and update accordingly
        let validator_config = client.validator_config(owner_account)?;
        let validator_address = if let Some(validator_address) = self.validator_address {
            validator_address
        } else {
            decode_validator_address(
                &validator_config.validator_network_address,
                &owner_account,
                &TEST_SHARED_VAL_NETADDR_KEY,
                0, // addr_idx
            )?
        };

        let fullnode_address = if let Some(fullnode_address) = self.fullnode_address {
            fullnode_address
        } else {
            decode_address(&validator_config.full_node_network_address)?
        };

        let txn = self.validator_config.build_transaction(
            sequence_number,
            fullnode_address,
            validator_address,
            true,
        )?;
        client.submit_transaction(txn.as_signed_user_txn().unwrap().clone())
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateKey {
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
}

impl RotateKey {
    pub fn execute(
        self,
        key_name: &'static str,
    ) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        let config = self
            .validator_config
            .config
            .load()?
            .override_json_server(&self.json_server)
            .override_validator_backend(
                &self.validator_config.validator_backend.validator_backend,
            )?;

        let mut storage = config.validator_backend();
        let key = storage.rotate_key(key_name)?;

        let set_validator_config = SetValidatorConfig {
            json_server: self.json_server,
            validator_config: self.validator_config,
            validator_address: None,
            fullnode_address: None,
        };

        set_validator_config.execute().map(|txn_ctx| (txn_ctx, key))
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateConsensusKey {
    #[structopt(flatten)]
    rotate_key: RotateKey,
}

impl RotateConsensusKey {
    pub fn execute(self) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        self.rotate_key.execute(CONSENSUS_KEY)
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateValidatorNetworkKey {
    #[structopt(flatten)]
    rotate_key: RotateKey,
}

impl RotateValidatorNetworkKey {
    pub fn execute(self) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        let (txn_ctx, key) = self.rotate_key.execute(VALIDATOR_NETWORK_KEY)?;
        Ok((txn_ctx, to_x25519(key)?))
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateFullNodeNetworkKey {
    #[structopt(flatten)]
    rotate_key: RotateKey,
}

impl RotateFullNodeNetworkKey {
    pub fn execute(self) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        let (txn_ctx, key) = self.rotate_key.execute(FULLNODE_NETWORK_KEY)?;
        Ok((txn_ctx, to_x25519(key)?))
    }
}

pub fn decode_validator_address(
    address: &RawEncNetworkAddress,
    account: &AccountAddress,
    key: &Key,
    addr_idx: u32,
) -> Result<NetworkAddress, Error> {
    let enc_addr = EncNetworkAddress::try_from(address).map_err(|e| {
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
    decode_address(&raw_addr)
}

pub fn decode_address(raw_address: &RawNetworkAddress) -> Result<NetworkAddress, Error> {
    let network_address = NetworkAddress::try_from(raw_address).map_err(|e| {
        Error::UnexpectedError(format!(
            "Failed to decode network address {}",
            e.to_string()
        ))
    })?;
    let protocols = network_address
        .as_slice()
        .iter()
        .filter(|protocol| match protocol {
            Protocol::Dns(_)
            | Protocol::Dns4(_)
            | Protocol::Dns6(_)
            | Protocol::Ip4(_)
            | Protocol::Ip6(_)
            | Protocol::Memory(_)
            | Protocol::Tcp(_) => true,
            _ => false,
        })
        .cloned()
        .collect::<Vec<_>>();
    Ok(NetworkAddress::try_from(protocols).unwrap())
}

#[derive(Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long, help = "Validator account address to display the config")]
    account_address: AccountAddress,
    #[structopt(flatten)]
    config: libra_management::config::ConfigPath,
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
}

impl ValidatorConfig {
    pub fn execute(self) -> Result<DecryptedValidatorConfig, Error> {
        let config = self.config.load()?.override_json_server(&self.json_server);
        let client = JsonRpcClientWrapper::new(config.json_server);
        client.validator_config(self.account_address).map(|config| {
            DecryptedValidatorConfig::from_validator_config(
                self.account_address,
                &TEST_SHARED_VAL_NETADDR_KEY,
                0,
                &config,
            )
            .unwrap()
        })
    }
}

#[derive(Serialize)]
pub struct DecryptedValidatorConfig {
    pub consensus_public_key: Ed25519PublicKey,
    pub validator_network_key: x25519::PublicKey,
    pub validator_network_address: NetworkAddress,
    pub full_node_network_key: x25519::PublicKey,
    pub full_node_network_address: NetworkAddress,
}

impl DecryptedValidatorConfig {
    pub fn from_validator_config(
        account: AccountAddress,
        address_encryption_key: &libra_network_address::encrypted::Key,
        addr_idx: u32,
        validator_config: &libra_types::validator_config::ValidatorConfig,
    ) -> Result<DecryptedValidatorConfig, Error> {
        let full_node_network_address =
            crate::validator_config::decode_address(&validator_config.full_node_network_address)?;

        let validator_network_address = crate::validator_config::decode_validator_address(
            &validator_config.validator_network_address,
            &account,
            address_encryption_key,
            addr_idx,
        )?;

        Ok(DecryptedValidatorConfig {
            consensus_public_key: validator_config.consensus_public_key.clone(),
            validator_network_key: validator_config.validator_network_identity_public_key,
            validator_network_address,
            full_node_network_key: validator_config.full_node_network_identity_public_key,
            full_node_network_address,
        })
    }
}
