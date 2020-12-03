// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{auto_validate::AutoValidate, json_rpc::JsonRpcClientWrapper, TransactionContext};
use diem_crypto::{ed25519::Ed25519PublicKey, x25519};
use diem_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, OPERATOR_ACCOUNT, OWNER_ACCOUNT, VALIDATOR_NETWORK_KEY,
};
use diem_management::{error::Error, secure_backend::ValidatorBackend, storage::to_x25519};
use diem_network_address::{NetworkAddress, Protocol};
use diem_network_address_encryption::Encryptor;
use diem_types::account_address::AccountAddress;
use serde::Serialize;
use std::{convert::TryFrom, str::FromStr};
use structopt::StructOpt;

// TODO: Load all chain IDs from the host
#[derive(Debug, StructOpt)]
pub struct SetValidatorConfig {
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_config: diem_management::validator_config::ValidatorConfig,
    #[structopt(
        long,
        required_unless = "fullnode-address",
        help = "Validator Network Address"
    )]
    validator_address: Option<NetworkAddress>,
    #[structopt(
        long,
        required_unless = "validator-address",
        help = "Full Node Network Address"
    )]
    fullnode_address: Option<NetworkAddress>,
    #[structopt(flatten)]
    auto_validate: AutoValidate,
    #[structopt(long, help = "Disables network address validation")]
    disable_address_validation: bool,
}

impl SetValidatorConfig {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let config = self
            .validator_config
            .config
            .load()?
            .override_chain_id(self.validator_config.chain_id)
            .override_json_server(&self.json_server)
            .override_validator_backend(
                &self.validator_config.validator_backend.validator_backend,
            )?;
        let storage = config.validator_backend();

        let operator_account = storage.account_address(OPERATOR_ACCOUNT)?;
        let owner_account = storage.account_address(OWNER_ACCOUNT)?;
        let encryptor = config.validator_backend().encryptor();
        let client = JsonRpcClientWrapper::new(config.json_server.clone());
        let sequence_number = client.sequence_number(operator_account)?;

        // See if the validator is in the set
        let in_set = client
            .validator_set(None)?
            .iter()
            .any(|vi| vi.account_address() == &owner_account);

        let validator_config = if in_set {
            // Retrieve the current validator / fullnode addresses and update accordingly
            let vc = client.validator_config(owner_account)?;
            DecryptedValidatorConfig::from_validator_config_resource(&vc, owner_account, &encryptor)
                .map(Some)
                .map_err(|e| {
                    Error::UnexpectedError(format!("Error parsing validator config: {}", e))
                })?
        } else {
            None
        };

        let validator_address = if let Some(validator_address) = &self.validator_address {
            validator_address.clone()
        } else if let Some(vc) = &validator_config {
            strip_address(&vc.validator_network_address)
        } else {
            return Err(Error::UnexpectedError(
                "Missing validator-network-address".to_string(),
            ));
        };

        let fullnode_address = if let Some(fullnode_address) = &self.fullnode_address {
            fullnode_address.clone()
        } else if let Some(vc) = &validator_config {
            strip_address(&vc.fullnode_network_address)
        } else {
            return Err(Error::UnexpectedError(
                "Missing fullnode-network-address".to_string(),
            ));
        };

        let txn = self.validator_config.build_transaction(
            sequence_number,
            fullnode_address,
            validator_address,
            validator_config.is_some(),
            self.disable_address_validation,
        )?;
        let mut transaction_context =
            client.submit_transaction(txn.as_signed_user_txn().unwrap().clone())?;

        // Perform auto validation if required
        transaction_context = self
            .auto_validate
            .execute(config.json_server, transaction_context)?;

        Ok(transaction_context)
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateKey {
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_config: diem_management::validator_config::ValidatorConfig,
    #[structopt(flatten)]
    auto_validate: AutoValidate,
}

impl RotateKey {
    pub fn execute(
        self,
        key_name: &'static str,
    ) -> Result<(TransactionContext, Ed25519PublicKey), Error> {
        // Load the config, storage backend and create a json rpc client.
        let config = self
            .validator_config
            .config()?
            .override_json_server(&self.json_server);
        let mut storage = config.validator_backend();
        let encryptor = config.validator_backend().encryptor();
        let client = JsonRpcClientWrapper::new(config.json_server.clone());

        // Fetch the current on-chain validator config for the node
        let owner_account = storage.account_address(OWNER_ACCOUNT)?;
        let validator_config = client.validator_config(owner_account).and_then(|vc| {
            DecryptedValidatorConfig::from_validator_config_resource(&vc, owner_account, &encryptor)
        })?;

        // Check that the key held in storage matches the key registered on-chain in the validator
        // config. If so, rotate the key in storage. If not, don't rotate the key in storage and
        // rather allow the next step to resubmit the set_validator_config transaction with the
        // current key (to resynchronize the validator config on the blockchain).
        let mut storage_key = storage.ed25519_public_from_private(key_name)?;
        let keys_match = match key_name {
            CONSENSUS_KEY => storage_key == validator_config.consensus_public_key,
            VALIDATOR_NETWORK_KEY => {
                Some(to_x25519(storage_key.clone())?)
                    == validator_config
                        .validator_network_address
                        .find_noise_proto()
            }
            FULLNODE_NETWORK_KEY => {
                Some(to_x25519(storage_key.clone())?)
                    == validator_config.fullnode_network_address.find_noise_proto()
            }
            _ => {
                return Err(Error::UnexpectedError(
                    "Rotate key was called with an unknown key name!".into(),
                ));
            }
        };
        if keys_match {
            storage_key = storage.rotate_key(key_name)?;
        }

        // Create and set the validator config state on the blockchain.
        let set_validator_config = SetValidatorConfig {
            json_server: self.json_server.clone(),
            validator_config: self.validator_config.clone(),
            validator_address: None,
            fullnode_address: None,
            auto_validate: self.auto_validate.clone(),
            disable_address_validation: true,
        };
        let mut transaction_context = set_validator_config.execute()?;

        // Perform auto validation if required
        transaction_context = self
            .auto_validate
            .execute(config.json_server, transaction_context)?;

        Ok((transaction_context, storage_key))
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

/// Returns only the IP/DNS + Port portion of the NetworkAddress
pub fn strip_address(address: &NetworkAddress) -> NetworkAddress {
    let protocols = address
        .as_slice()
        .iter()
        .filter(|protocol| {
            matches!(protocol,
            Protocol::Dns(_)
            | Protocol::Dns4(_)
            | Protocol::Dns6(_)
            | Protocol::Ip4(_)
            | Protocol::Ip6(_)
            | Protocol::Memory(_)
            | Protocol::Tcp(_))
        })
        .cloned()
        .collect::<Vec<_>>();
    NetworkAddress::try_from(protocols).unwrap()
}

#[derive(Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long, help = "Validator account address to display the config")]
    account_address: AccountAddress,
    #[structopt(flatten)]
    config: diem_management::config::ConfigPath,
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl ValidatorConfig {
    pub fn execute(self) -> Result<DecryptedValidatorConfig, Error> {
        let config = self
            .config
            .load()?
            .override_json_server(&self.json_server)
            .override_validator_backend(&self.validator_backend.validator_backend)?;
        let encryptor = config.validator_backend().encryptor();
        let client = JsonRpcClientWrapper::new(config.json_server);
        client
            .validator_config(self.account_address)
            .and_then(|vc| {
                DecryptedValidatorConfig::from_validator_config_resource(
                    &vc,
                    self.account_address,
                    &encryptor,
                )
            })
    }
}

#[derive(Serialize)]
pub struct DecryptedValidatorConfig {
    pub name: String,
    pub consensus_public_key: Ed25519PublicKey,
    pub validator_network_address: NetworkAddress,
    pub fullnode_network_address: NetworkAddress,
}

impl DecryptedValidatorConfig {
    pub fn from_validator_config_resource(
        config_resource: &diem_types::validator_config::ValidatorConfigResource,
        account_address: AccountAddress,
        encryptor: &Encryptor,
    ) -> Result<Self, Error> {
        let config = config_resource.validator_config.as_ref().ok_or_else(|| {
            Error::JsonRpcReadError("validator-config", "not present".to_string())
        })?;

        let mut value = Self::from_validator_config(&config, account_address, encryptor)?;
        value.name = Self::human_name(&config_resource.human_name);
        Ok(value)
    }

    pub fn from_validator_config(
        config: &diem_types::validator_config::ValidatorConfig,
        account_address: AccountAddress,
        encryptor: &Encryptor,
    ) -> Result<Self, Error> {
        let fullnode_network_addresses = config
            .fullnode_network_addresses()
            .map_err(|e| Error::NetworkAddressDecodeError(e.to_string()))?;

        let validator_network_addresses = encryptor
            .decrypt(&config.validator_network_addresses, account_address)
            .unwrap_or_else(|error| {
                println!(
                    "Unable to decode network address for account {}: {}. Using a dummy validator network address!",
                    account_address, error
                );
                vec![NetworkAddress::from_str("/dns4/could-not-decrypt").unwrap()]
            });

        Ok(DecryptedValidatorConfig {
            name: "".to_string(),
            consensus_public_key: config.consensus_public_key.clone(),
            fullnode_network_address: fullnode_network_addresses[0].clone(),
            validator_network_address: validator_network_addresses[0].clone(),
        })
    }

    pub fn human_name(name: &[u8]) -> String {
        std::str::from_utf8(name)
            .map(|v| v.to_string())
            .unwrap_or_else(|_| hex::encode(name))
    }
}
