// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    json_rpc::JsonRpcClientWrapper,
    validator_config::{fullnode_addresses, validator_addresses, DecryptedValidatorConfig},
};
use diem_crypto::ed25519::Ed25519PublicKey;
use diem_management::{config::ConfigPath, error::Error, secure_backend::ValidatorBackend};
use diem_network_address_encryption::Encryptor;
use diem_secure_storage::Storage;
use diem_types::{
    account_address::AccountAddress, network_address::NetworkAddress, validator_info::ValidatorInfo,
};
use serde::Serialize;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidatorSet {
    #[structopt(flatten)]
    config: ConfigPath,
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(long, help = "AccountAddress to retrieve the validator set info")]
    account_address: Option<AccountAddress>,
    #[structopt(
        long,
        help = "The secure backend that contains the network address encryption keys"
    )]
    validator_backend: Option<ValidatorBackend>,
}

impl ValidatorSet {
    pub fn execute(self) -> Result<Vec<DecryptedValidatorInfo>, Error> {
        let mut config = self.config.load()?.override_json_server(&self.json_server);
        let client = JsonRpcClientWrapper::new(config.clone().json_server);

        let encryptor = if let Some(backend) = &self.validator_backend {
            config = config.override_validator_backend(&backend.validator_backend)?;
            config.validator_backend().encryptor()
        } else {
            Encryptor::empty()
        };

        decode_validator_set(encryptor, client, self.account_address)
    }
}

pub fn decode_validator_set(
    encryptor: Encryptor<Storage>,
    client: JsonRpcClientWrapper,
    account_address: Option<AccountAddress>,
) -> Result<Vec<DecryptedValidatorInfo>, Error> {
    let set = client.validator_set(account_address)?;

    let mut decoded_set = Vec::new();
    for info in set {
        let config = DecryptedValidatorConfig::from_validator_config(
            info.config(),
            *info.account_address(),
            &encryptor,
        )
        .map_err(|e| Error::NetworkAddressDecodeError(e.to_string()))?;

        let config_resource = client.validator_config(*info.account_address())?;
        let name = DecryptedValidatorConfig::human_name(&config_resource.human_name);

        let info = DecryptedValidatorInfo {
            name,
            account_address: *info.account_address(),
            consensus_public_key: config.consensus_public_key,
            fullnode_network_address: config.fullnode_network_address,
            validator_network_address: config.validator_network_address,
        };
        decoded_set.push(info);
    }

    Ok(decoded_set)
}

pub fn validator_set_full_node_addresses(
    client: JsonRpcClientWrapper,
    account_address: Option<AccountAddress>,
) -> Result<Vec<(String, AccountAddress, Vec<NetworkAddress>)>, Error> {
    validator_set_addresses(client, account_address, |info| {
        fullnode_addresses(info.config())
    })
}

pub fn validator_set_validator_addresses(
    client: JsonRpcClientWrapper,
    encryptor: &Encryptor<Storage>,
    account_address: Option<AccountAddress>,
) -> Result<Vec<(String, AccountAddress, Vec<NetworkAddress>)>, Error> {
    validator_set_addresses(client, account_address, |info| {
        validator_addresses(info.config(), *info.account_address(), encryptor)
    })
}

fn validator_set_addresses<F: Fn(ValidatorInfo) -> Result<Vec<NetworkAddress>, Error>>(
    client: JsonRpcClientWrapper,
    account_address: Option<AccountAddress>,
    address_accessor: F,
) -> Result<Vec<(String, AccountAddress, Vec<NetworkAddress>)>, Error> {
    let set = client.validator_set(account_address)?;
    let mut decoded_set = Vec::new();
    for info in set {
        let config_resource = client.validator_config(*info.account_address())?;
        let name = DecryptedValidatorConfig::human_name(&config_resource.human_name);
        let peer_id = *info.account_address();
        let addrs = address_accessor(info)?;
        decoded_set.push((name, peer_id, addrs));
    }

    Ok(decoded_set)
}
#[derive(Serialize)]
pub struct DecryptedValidatorInfo {
    pub name: String,
    pub account_address: AccountAddress,
    pub consensus_public_key: Ed25519PublicKey,
    pub fullnode_network_address: NetworkAddress,
    pub validator_network_address: NetworkAddress,
}
