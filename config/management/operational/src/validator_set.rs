// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, validator_config::DecryptedValidatorConfig};
use libra_management::{config::ConfigPath, error::Error};
use libra_network_address::encrypted::{Key, TEST_SHARED_VAL_NETADDR_KEY};
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
    // The validator's account address. AccountAddresses are initially derived from the account
    // auth pubkey; however, the auth key can be rotated, so one should not rely on this
    // initial property.
    pub account_address: AccountAddress,
    // Voting power of this validator
    pub consensus_voting_power: u64,
    // Validator config
    pub config: DecryptedValidatorConfig,
}

impl DecryptedValidatorInfo {
    fn from_validator_info(
        validator_encryption_key: &Key,
        addr_idx: u32,
        validator_info: &ValidatorInfo,
    ) -> Result<DecryptedValidatorInfo, Error> {
        let account = *validator_info.account_address();
        Ok(DecryptedValidatorInfo {
            account_address: account,
            consensus_voting_power: validator_info.consensus_voting_power(),
            config: DecryptedValidatorConfig::from_validator_config(
                account,
                validator_encryption_key,
                addr_idx,
                validator_info.config(),
            )?,
        })
    }
}
