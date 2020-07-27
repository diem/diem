// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_rpc::JsonRpcClientWrapper;
use libra_management::{config::ConfigPath, error::Error};
use libra_types::{account_address::AccountAddress, validator_info::ValidatorInfo};
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
    pub fn execute(self) -> Result<Vec<ValidatorInfo>, Error> {
        let config = self.config.load()?.override_json_server(&self.json_server);
        let client = JsonRpcClientWrapper::new(config.json_server);
        client.validator_set(self.account_address)
    }
}
