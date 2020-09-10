// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::json_rpc::JsonRpcClientWrapper;
use libra_management::{config::ConfigPath, error::Error};
use libra_secure_json_rpc::VMStatusView;
use libra_types::account_address::AccountAddress;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidateTransaction {
    #[structopt(flatten)]
    config: ConfigPath,
    /// JSON-RPC Endpoint (e.g. http://localhost:8080)
    #[structopt(long, required_unless = "config")]
    json_server: Option<String>,
    #[structopt(long, help = "AccountAddress to check transactions")]
    account_address: AccountAddress,
    #[structopt(long, help = "Sequence number to verify")]
    sequence_number: u64,
}

/// Returns `true` if we've passed by the expected sequence number
impl ValidateTransaction {
    pub fn execute(self) -> Result<Option<VMStatusView>, Error> {
        let config = self.config.load()?.override_json_server(&self.json_server);
        JsonRpcClientWrapper::new(config.json_server)
            .transaction_status(self.account_address, self.sequence_number)
    }
}
