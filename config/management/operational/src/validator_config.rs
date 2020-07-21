// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::x25519;
use libra_global_constants::{OWNER_ACCOUNT, VALIDATOR_NETWORK_KEY};
use libra_management::{
    error::Error, json_rpc::JsonRpcClientWrapper, storage::StorageWrapper, TransactionContext,
};
use libra_network_address::{
    encrypted::{EncNetworkAddress, Key, RawEncNetworkAddress, TEST_SHARED_VAL_NETADDR_KEY},
    NetworkAddress, Protocol, RawNetworkAddress,
};
use libra_types::account_address::AccountAddress;
use std::convert::TryFrom;
use structopt::StructOpt;

// TODO: Load all chain IDs from the host
#[derive(Debug, StructOpt)]
pub struct SetValidatorConfig {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080")]
    host: String,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
    #[structopt(long, help = "Validator Network Address")]
    validator_address: Option<NetworkAddress>,
    #[structopt(long, help = "Full Node Network Address")]
    fullnode_address: Option<NetworkAddress>,
}

impl SetValidatorConfig {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let storage = StorageWrapper::new(
            self.validator_config.validator_backend.name(),
            &self.validator_config.validator_backend.validator_backend,
        )?;
        let owner_account = storage.account_address(OWNER_ACCOUNT)?;

        let client = JsonRpcClientWrapper::new(self.host);
        let sequence_number = client.sequence_number(owner_account)?;

        // Retrieve the current validator / fullnode addresses and update accordingly
        let validator_config = client.validator_config(owner_account)?;
        let validator_address = if let Some(validator_address) = self.validator_address {
            validator_address
        } else {
            decode_validator_address(
                validator_config.validator_network_address,
                &owner_account,
                &TEST_SHARED_VAL_NETADDR_KEY,
                0, // addr_idx
            )?
        };

        let fullnode_address = if let Some(fullnode_address) = self.fullnode_address {
            fullnode_address
        } else {
            decode_address(validator_config.full_node_network_address)?
        };

        let txn = self.validator_config.build_transaction(
            sequence_number,
            fullnode_address,
            validator_address,
            true,
        )?;
        client.submit_transaction(txn.as_signed_user_txn().unwrap().clone())
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateValidatorNetworkKey {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080)")]
    host: String,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
}

impl RotateValidatorNetworkKey {
    pub fn execute(self) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        let mut storage = StorageWrapper::new(
            self.validator_config.validator_backend.name(),
            &self.validator_config.validator_backend.validator_backend,
        )?;
        let key = storage.rotate_key(VALIDATOR_NETWORK_KEY)?;
        let key = StorageWrapper::x25519(key)?;

        let set_validator_config = SetValidatorConfig {
            host: self.host,
            validator_config: self.validator_config,
            validator_address: None,
            fullnode_address: None,
        };

        set_validator_config.execute().map(|txn_ctx| (txn_ctx, key))
    }
}

fn decode_validator_address(
    address: RawEncNetworkAddress,
    account: &AccountAddress,
    key: &Key,
    addr_idx: u32,
) -> Result<NetworkAddress, Error> {
    let enc_addr = EncNetworkAddress::try_from(&address).map_err(|e| {
        Error::UnexpectedError(format!(
            "Failed to decode network address {}",
            e.to_string()
        ))
    })?;
    let raw_addr = enc_addr.decrypt(key, account, addr_idx).map_err(|e| {
        Error::UnexpectedError(format!(
            "Failed to decrypt network address {}",
            e.to_string()
        ))
    })?;
    decode_address(raw_addr)
}

fn decode_address(raw_address: RawNetworkAddress) -> Result<NetworkAddress, Error> {
    let network_address = NetworkAddress::try_from(&raw_address).map_err(|e| {
        Error::UnexpectedError(format!(
            "Failed to decode network address {}",
            e.to_string()
        ))
    })?;
    let protocols = network_address
        .as_slice()
        .iter()
        .filter(|protocol| match protocol {
            Protocol::Dns(_)
            | Protocol::Dns4(_)
            | Protocol::Dns6(_)
            | Protocol::Ip4(_)
            | Protocol::Ip6(_)
            | Protocol::Memory(_)
            | Protocol::Tcp(_) => true,
            _ => false,
        })
        .cloned()
        .collect::<Vec<_>>();
    Ok(NetworkAddress::try_from(protocols).unwrap())
}
