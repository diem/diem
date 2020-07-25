// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_rpc::JsonRpcClientWrapper;
use libra_management::error::Error;
use libra_types::{account_address::AccountAddress, validator_info::ValidatorInfo};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidatorSet {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080")]
    json_server: String,
    #[structopt(long, help = "AccountAddress to retrieve the validator set info")]
    account_address: Option<AccountAddress>,
}

impl ValidatorSet {
    pub fn execute(self) -> Result<Vec<ValidatorInfo>, Error> {
        let client = JsonRpcClientWrapper::new(self.json_server);
        client.validator_set(self.account_address)
    }
}
