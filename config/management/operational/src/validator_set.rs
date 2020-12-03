// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, validator_config::DecryptedValidatorConfig};
use diem_crypto::ed25519::Ed25519PublicKey;
use diem_management::{config::ConfigPath, error::Error, secure_backend::ValidatorBackend};
use diem_network_address::NetworkAddress;
use diem_types::account_address::AccountAddress;
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
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl ValidatorSet {
    pub fn execute(self) -> Result<Vec<DecryptedValidatorInfo>, Error> {
        let config = self
            .config
            .load()?
            .override_json_server(&self.json_server)
            .override_validator_backend(&self.validator_backend.validator_backend)?;
        let encryptor = config.validator_backend().encryptor();
        let client = JsonRpcClientWrapper::new(config.json_server);
        let set = client.validator_set(self.account_address)?;

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
}

#[derive(Serialize)]
pub struct DecryptedValidatorInfo {
    pub name: String,
    pub account_address: AccountAddress,
    pub consensus_public_key: Ed25519PublicKey,
    pub fullnode_network_address: NetworkAddress,
    pub validator_network_address: NetworkAddress,
}
