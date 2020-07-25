// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, TransactionContext};
use libra_secure_json_rpc::{JsonRpcClient, VMStatusView};
use libra_types::{
    account_address::AccountAddress, account_config::AccountResource, account_state::AccountState,
    transaction::SignedTransaction, validator_config::ValidatorConfig,
};

/// A wrapper around JSON RPC for error handling
pub struct JsonRpcClientWrapper {
    client: JsonRpcClient,
}

impl JsonRpcClientWrapper {
    pub fn new(host: String) -> JsonRpcClientWrapper {
        JsonRpcClientWrapper {
            client: JsonRpcClient::new(host),
        }
    }

    pub fn submit_transaction(
        &self,
        transaction: SignedTransaction,
    ) -> Result<TransactionContext, Error> {
        self.client
            .submit_transaction(transaction.clone())
            .map_err(|e| Error::JsonRpcWriteError("transaction", e.to_string()))?;
        Ok(TransactionContext::new(
            transaction.sender(),
            transaction.sequence_number(),
        ))
    }

    pub fn account_state(&self, account: AccountAddress) -> Result<AccountState, Error> {
        self.client
            .get_account_state(account, None)
            .map_err(|e| Error::JsonRpcReadError("account-state", e.to_string()))
    }

    pub fn validator_config(&self, account: AccountAddress) -> Result<ValidatorConfig, Error> {
        resource(
            "validator-config-resource",
            self.account_state(account)?.get_validator_config_resource(),
        )?
        .validator_config
        .ok_or_else(|| Error::JsonRpcReadError("validator-config", "not present".to_string()))
    }

    pub fn account_resource(&self, account: AccountAddress) -> Result<AccountResource, Error> {
        let account_state = self.account_state(account)?;
        resource("account-resource", account_state.get_account_resource())
    }

    pub fn sequence_number(&self, account: AccountAddress) -> Result<u64, Error> {
        Ok(self.account_resource(account)?.sequence_number())
    }

    pub fn transaction_status(
        &self,
        account: AccountAddress,
        sequence_number: u64,
    ) -> Result<Option<VMStatusView>, Error> {
        self.client
            .get_transaction_status(account, sequence_number)
            .map(|maybe_txn_status| maybe_txn_status.map(|status| status.vm_status))
            .map_err(|e| Error::JsonRpcReadError("transaction-status", e.to_string()))
    }
}

fn resource<T>(
    resource_name: &'static str,
    maybe_resource: Result<Option<T>, anyhow::Error>,
) -> Result<T, Error> {
    match maybe_resource {
        Ok(Some(resource)) => Ok(resource),
        Ok(None) => Err(Error::JsonRpcReadError(
            resource_name,
            "not present".to_string(),
        )),
        Err(e) => Err(Error::JsonRpcReadError(resource_name, e.to_string())),
    }
}
