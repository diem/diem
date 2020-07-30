// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, validator_config::DecryptedValidatorConfig};
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_management::{config::ConfigPath, error::Error};
use libra_network_address::{
    encrypted::{Key, TEST_SHARED_VAL_NETADDR_KEY},
    NetworkAddress,
};
use libra_types::{account_address::AccountAddress, validator_info::ValidatorInfo};
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
}

impl ValidatorSet {
    pub fn execute(self) -> Result<Vec<DecryptedValidatorInfo>, Error> {
        let config = self.config.load()?.override_json_server(&self.json_server);
        let client = JsonRpcClientWrapper::new(config.json_server);
        client.validator_set(self.account_address).map(|vec| {
            vec.iter()
                .map(|info| {
                    DecryptedValidatorInfo::from_validator_info(
                        &TEST_SHARED_VAL_NETADDR_KEY,
                        0,
                        info,
                    )
                    .unwrap()
                })
                .collect()
        })
    }
}

#[derive(Serialize)]
pub struct DecryptedValidatorInfo {
    pub account_address: AccountAddress,
    // Validator config
    pub consensus_public_key: Ed25519PublicKey,
    pub full_node_network_key: x25519::PublicKey,
    pub full_node_network_address: NetworkAddress,
    pub validator_network_key: x25519::PublicKey,
    pub validator_network_address: NetworkAddress,
}

impl DecryptedValidatorInfo {
    fn from_validator_info(
        validator_encryption_key: &Key,
        addr_idx: u32,
        validator_info: &ValidatorInfo,
    ) -> Result<DecryptedValidatorInfo, Error> {
        let account = *validator_info.account_address();
        let config = DecryptedValidatorConfig::from_validator_config(
            account,
            validator_encryption_key,
            addr_idx,
            validator_info.config(),
        )?;

        Ok(DecryptedValidatorInfo {
            account_address: account,
            consensus_public_key: config.consensus_public_key,
            full_node_network_key: config.full_node_network_key,
            full_node_network_address: config.full_node_network_address,
            validator_network_key: config.validator_network_key,
            validator_network_address: config.validator_network_address,
        })
    }
}
