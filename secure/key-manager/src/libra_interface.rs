// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use libra_secure_json_rpc::JsonRpcClient;
use libra_types::{
    account_address::AccountAddress, account_config, account_state::AccountState,
    transaction::Transaction, validator_config::ValidatorConfig, validator_info::ValidatorInfo,
};

/// This defines a generic trait used to interact with the Libra blockchain. In production, this
/// will be talking to a JSON-RPC service. For tests, this may be an executor and storage directly.
pub trait LibraInterface {
    /// Retrieves the current time from the blockchain, this is returned as microseconds.
    fn libra_timestamp(&self) -> Result<u64, Error>;

    /// Retrieves the last reconfiguration time from the blockchain, this is returned as
    /// microseconds.
    fn last_reconfiguration(&self) -> Result<u64, Error>;

    /// Retrieve current sequence number for the provided account.
    fn retrieve_sequence_number(&self, account: AccountAddress) -> Result<u64, Error>;

    /// Submits a transaction to the block chain and returns successfully if the transaction was
    /// successfully submitted. It does not necessarily mean the transaction successfully executed.
    fn submit_transaction(&self, transaction: Transaction) -> Result<(), Error>;

    /// Retrieves the ValidatorConfig at the specified AccountAddress if one exists.
    fn retrieve_validator_config(&self, account: AccountAddress) -> Result<ValidatorConfig, Error>;

    /// Retrieves the ValidatorInfo for the specified account from the current ValidatorSet if one
    /// exists.
    fn retrieve_validator_info(&self, account: AccountAddress) -> Result<ValidatorInfo, Error>;

    /// Fetches the AccountState associated with a specific account. This is currently only
    /// used by test code, but it's not completely inconceivable that non-test code will want
    /// access to this in the future.
    fn retrieve_account_state(&self, account: AccountAddress) -> Result<AccountState, Error>;
}

/// This implements the LibraInterface by talking directly to the JSON RPC API.
///
/// DISCLAIMER: this implementation assumes that the json rpc client explicitly trusts the json rpc
/// server that is responding to its requests (e.g., the client assumes the server has already been
/// authenticated, provides encrypted and freshness protected messages, etc.). As such, the security
/// of the server will need to be verified in production before this interface should be used.
/// Pointing the client to an untrusted (and potentially malicious) json rpc server, can result in
/// denial-of-service attacks (e.g., against the key manager).
///
/// TODO(joshlind): add proof checks to the JsonRpcClient to verify the state returned by the json
/// rpc server we're talking to. Although we won't be able to guarantee freshness, it's better than
/// simply trusting the response for correctness..
#[derive(Clone)]
pub struct JsonRpcLibraInterface {
    client: JsonRpcClient,
}

impl JsonRpcLibraInterface {
    pub fn new(client: JsonRpcClient) -> Self {
        Self { client }
    }
}

impl LibraInterface for JsonRpcLibraInterface {
    fn libra_timestamp(&self) -> Result<u64, Error> {
        let account = account_config::association_address();
        let libra_timestamp_resource = self
            .retrieve_account_state(account)?
            .get_libra_timestamp_resource();

        match libra_timestamp_resource {
            Ok(timestamp_resource) => timestamp_resource
                .map(|timestamp_resource| timestamp_resource.libra_timestamp.microseconds)
                .ok_or_else(|| {
                    Error::DataDoesNotExist(format!(
                        "LibraTimestampResource not found for account: {:?}",
                        account
                    ))
                }),
            e => Err(Error::UnknownError(format!("{:?}", e))),
        }
    }

    fn last_reconfiguration(&self) -> Result<u64, Error> {
        let account = account_config::association_address();
        let configuration_resource = self
            .retrieve_account_state(account)?
            .get_configuration_resource();

        match configuration_resource {
            Ok(config_resource) => config_resource
                .map(|config_resource| config_resource.last_reconfiguration_time())
                .ok_or_else(|| {
                    Error::DataDoesNotExist(format!(
                        "ConfigurationResource not found for account: {:?}",
                        account
                    ))
                }),
            e => Err(Error::UnknownError(format!("{:?}", e))),
        }
    }

    fn retrieve_sequence_number(&self, account: AccountAddress) -> Result<u64, Error> {
        let account_resource = self.retrieve_account_state(account)?.get_account_resource();

        match account_resource {
            Ok(account_resource) => account_resource
                .map(|account_resource| account_resource.sequence_number())
                .ok_or_else(|| {
                    Error::DataDoesNotExist(format!(
                        "AccountResource not found for account: {:?}",
                        account
                    ))
                }),
            e => Err(Error::UnknownError(format!("{:?}", e))),
        }
    }

    fn submit_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        if let Transaction::UserTransaction(signed_txn) = transaction {
            self.client
                .submit_signed_transaction(signed_txn)
                .map_err(|e| {
                    Error::UnknownError(format!(
                        "Failed to submit signed transaction. Error: {:?}",
                        e,
                    ))
                })
        } else {
            Err(Error::UnknownError(format!(
                "Unable to submit a transaction type that is not a SignedTransaction: {:?}",
                transaction
            )))
        }
    }

    fn retrieve_validator_config(&self, account: AccountAddress) -> Result<ValidatorConfig, Error> {
        let validator_config_resource = self
            .retrieve_account_state(account)?
            .get_validator_config_resource();

        match validator_config_resource {
            Ok(config_resource) => config_resource
                .map(|config_resource| config_resource.validator_config)
                .ok_or_else(|| {
                    Error::DataDoesNotExist(format!(
                        "ValidatorConfigResource not found for account: {:?}",
                        account
                    ))
                }),
            e => Err(Error::UnknownError(format!("{:?}", e))),
        }
    }

    fn retrieve_validator_info(&self, account: AccountAddress) -> Result<ValidatorInfo, Error> {
        let validator_set_account = account_config::validator_set_address();
        let validator_set = self
            .retrieve_account_state(validator_set_account)?
            .get_validator_set();

        match validator_set {
            Ok(validator_set) => match validator_set {
                Some(validator_set) => validator_set
                    .payload()
                    .iter()
                    .find(|validator_info| validator_info.account_address() == &account)
                    .cloned()
                    .ok_or(Error::ValidatorInfoNotFound(account)),
                None => Err(Error::DataDoesNotExist(format!(
                    "ValidatorSet not found for account: {:?}",
                    account
                ))),
            },
            Err(e) => Err(Error::UnknownError(format!("{:?}", e))),
        }
    }

    fn retrieve_account_state(&self, account: AccountAddress) -> Result<AccountState, Error> {
        self.client.get_account_state(account, None).map_err(|e| {
            Error::UnknownError(format!(
                "Failed to get AccountState for account: {:?}. Error: {:?}",
                account, e
            ))
        })
    }
}
