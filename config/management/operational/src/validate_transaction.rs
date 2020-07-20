// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_management::{error::Error, json_rpc::JsonRpcClientWrapper};
use libra_types::account_address::AccountAddress;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct ValidateTransaction {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080")]
    host: String,
    #[structopt(long, help = "AccountAddress to check transactions")]
    account_address: AccountAddress,
    #[structopt(long, help = "Sequence number to verify")]
    sequence_number: u64,
}

/// Returns `true` if we've passed by the expected sequence number
impl ValidateTransaction {
    pub fn execute(self) -> Result<bool, Error> {
        let client = JsonRpcClientWrapper::new(self.host);
        let sequence_number = client.sequence_number(self.account_address)?;
        Ok(sequence_number >= self.sequence_number)
    }
}
