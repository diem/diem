// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc::JsonRpcClientWrapper, TransactionContext};
use diem_management::{config::ConfigPath, error::Error};
use diem_types::account_address::AccountAddress;
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
    pub fn new(json_server: String, account_address: AccountAddress, sequence_number: u64) -> Self {
        Self {
            config: Default::default(),
            json_server: Some(json_server),
            account_address,
            sequence_number,
        }
    }

    pub fn execute(&self) -> Result<TransactionContext, Error> {
        let config = self.config.load()?.override_json_server(&self.json_server);
        let vm_status = JsonRpcClientWrapper::new(config.json_server)
            .transaction_status(self.account_address, self.sequence_number)?;
        Ok(TransactionContext::new_with_validation(
            self.account_address,
            self.sequence_number,
            vm_status,
        ))
    }
}
