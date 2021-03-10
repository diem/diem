// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionContext;
use diem_client::{views::VMStatusView, BlockingClient};
use diem_management::error::Error;
use diem_types::{
    account_address::AccountAddress, account_config, account_config::AccountResource,
    account_state::AccountState, account_state_blob::AccountStateBlob,
    transaction::SignedTransaction, validator_config::ValidatorConfigResource,
    validator_info::ValidatorInfo,
};
use std::convert::TryFrom;

/// A wrapper around JSON RPC for error handling
pub struct JsonRpcClientWrapper {
    client: BlockingClient,
}

impl JsonRpcClientWrapper {
    pub fn new(host: String) -> JsonRpcClientWrapper {
        JsonRpcClientWrapper {
            client: BlockingClient::new(host),
        }
    }

    pub fn submit_transaction(
        &self,
        transaction: SignedTransaction,
    ) -> Result<TransactionContext, Error> {
        self.client
            .submit(&transaction)
            .map_err(|e| Error::JsonRpcWriteError("transaction", e.to_string()))?;
        Ok(TransactionContext::new(
            transaction.sender(),
            transaction.sequence_number(),
        ))
    }

    pub fn account_state(&self, account: AccountAddress) -> Result<AccountState, Error> {
        let account_state = self
            .client
            .get_account_state_with_proof(account, None, None)
            .map_err(|e| Error::JsonRpcReadError("account-state", e.to_string()))?
            .into_inner();

        let blob = account_state.blob.ok_or_else(|| {
            Error::JsonRpcReadError("account-state", "No Validator set".to_string())
        })?;
        let account_state_blob = AccountStateBlob::from(
            bcs::from_bytes::<Vec<u8>>(&blob)
                .map_err(|e| Error::JsonRpcReadError("account-state", e.to_string()))?,
        );
        let account_state = AccountState::try_from(&account_state_blob)
            .map_err(|e| Error::JsonRpcReadError("account-state", e.to_string()))?;
        Ok(account_state)
    }

    pub fn validator_config(
        &self,
        account: AccountAddress,
    ) -> Result<ValidatorConfigResource, Error> {
        resource(
            "validator-config-resource",
            self.account_state(account)?.get_validator_config_resource(),
        )
    }

    /// This method returns all validator infos currently registered in the validator set of the
    /// Diem blockchain. If account is specified, only a single validator info is returned: the
    /// one that matches the given account.
    pub fn validator_set(
        &self,
        account: Option<AccountAddress>,
    ) -> Result<Vec<ValidatorInfo>, Error> {
        let validator_set_account = account_config::validator_set_address();
        let validator_set = self
            .account_state(validator_set_account)?
            .get_validator_set();

        match validator_set {
            Ok(Some(validator_set)) => {
                let mut validator_infos = vec![];
                for validator_info in validator_set.payload().iter() {
                    if let Some(account) = account {
                        if validator_info.account_address() == &account {
                            validator_infos.push(validator_info.clone());
                        }
                    } else {
                        validator_infos.push(validator_info.clone());
                    }
                }

                if validator_infos.is_empty() {
                    return Err(Error::UnexpectedError(
                        "No validator sets were found!".to_string(),
                    ));
                }
                Ok(validator_infos)
            }
            Ok(None) => Err(Error::JsonRpcReadError(
                "validator-set",
                "not present".to_string(),
            )),
            Err(e) => Err(Error::JsonRpcReadError("validator-set", e.to_string())),
        }
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
            .get_account_transaction(account, sequence_number, false)
            .map(|maybe_txn_status| maybe_txn_status.into_inner().map(|status| status.vm_status))
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
