// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_management::{error::Error, json_rpc::JsonRpcClientWrapper};
use libra_types::account_address::AccountAddress;
use std::{fs, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct SubmitTransaction {
    #[structopt(long)]
    pub host: String,
    #[structopt(long)]
    pub transaction_path: PathBuf,
}

impl SubmitTransaction {
    pub fn execute(self) -> Result<(), Error> {
        let transaction_path = self.transaction_path.to_str().unwrap().to_string();
        let data = fs::read(&self.transaction_path)
            .map_err(|e| Error::UnableToReadFile(transaction_path.clone(), e.to_string()))?;
        let transaction = lcs::from_bytes(&data)
            .map_err(|e| Error::UnableToParseFile(transaction_path, e.to_string()))?;

        let client = JsonRpcClientWrapper::new(self.host);
        client.submit_transaction(transaction)?;
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
pub struct ReadAccountState {
    #[structopt(long)]
    pub host: String,
    #[structopt(long)]
    pub account: AccountAddress,
}

impl ReadAccountState {
    pub fn execute(self) -> Result<libra_types::account_state::AccountState, Error> {
        let client = JsonRpcClientWrapper::new(self.host);
        client.account_state(self.account)
    }
}
