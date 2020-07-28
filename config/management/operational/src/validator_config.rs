// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::x25519;
use libra_global_constants::{FULLNODE_NETWORK_KEY, OWNER_ACCOUNT, VALIDATOR_NETWORK_KEY};
use libra_management::{
    error::Error, json_rpc::JsonRpcClientWrapper, storage::StorageWrapper, TransactionContext,
};
use libra_network_address::{
    deserialize_addresses,
    encrypted::{
        self as netaddr, decrypt_addresses, RawEncNetworkAddress, TEST_SHARED_VAL_NETADDR_KEY,
        TEST_SHARED_VAL_NETADDR_KEY_VERSION,
    },
    NetworkAddress, Protocol, RawNetworkAddress,
};
use libra_types::account_address::AccountAddress;
use std::{collections::HashMap, convert::TryFrom, iter};
use structopt::StructOpt;

// TODO: Load all chain IDs from the host
#[derive(Debug, StructOpt)]
pub struct SetValidatorConfig {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080")]
    host: String,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
    #[structopt(long, help = "Validator Network Addresses")]
    validator_addresses: Vec<NetworkAddress>,
    #[structopt(long, help = "Full Node Network Addresses")]
    full_node_addresses: Vec<NetworkAddress>,
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

        let validator_addresses = self.validator_addresses;
        let full_node_addresses = self.full_node_addresses;

        let validator_config = client.validator_config(owner_account)?;
        let config_validator_addresses = validator_config.validator_network_addresses;
        let config_full_node_addresses = validator_config.full_node_network_addresses;

        // TODO(philiphayes): read actual shared_val_netaddr_keys from secure storage
        let shared_val_netaddr_keys: HashMap<_, _> = iter::once((
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            TEST_SHARED_VAL_NETADDR_KEY,
        ))
        .collect();

        // Retrieve the current validator / fullnode addresses and update accordingly
        let validator_addresses = if !validator_addresses.is_empty() {
            validator_addresses
        } else {
            decode_validator_addresses(
                &owner_account,
                &shared_val_netaddr_keys,
                &config_validator_addresses,
            )?
        };

        let full_node_addresses = if !full_node_addresses.is_empty() {
            full_node_addresses
        } else {
            decode_full_node_addresses(&owner_account, &config_full_node_addresses)?
        };

        let txn = self.validator_config.build_transaction(
            sequence_number,
            full_node_addresses,
            validator_addresses,
            true,
        )?;
        client.submit_transaction(txn.as_signed_user_txn().unwrap().clone())
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateNetworkKey {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080)")]
    host: String,
    #[structopt(flatten)]
    validator_config: libra_management::validator_config::ValidatorConfig,
}

impl RotateNetworkKey {
    pub fn execute(
        self,
        key_name: &'static str,
    ) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        let mut storage = StorageWrapper::new(
            self.validator_config.validator_backend.name(),
            &self.validator_config.validator_backend.validator_backend,
        )?;
        let key = storage.rotate_key(key_name)?;
        let key = StorageWrapper::x25519(key)?;

        let set_validator_config = SetValidatorConfig {
            host: self.host,
            validator_config: self.validator_config,
            validator_addresses: Vec::new(),
            full_node_addresses: Vec::new(),
        };

        set_validator_config.execute().map(|txn_ctx| (txn_ctx, key))
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateValidatorNetworkKey {
    #[structopt(flatten)]
    rotate_network_key: RotateNetworkKey,
}

impl RotateValidatorNetworkKey {
    pub fn execute(self) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        self.rotate_network_key.execute(VALIDATOR_NETWORK_KEY)
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateFullNodeNetworkKey {
    #[structopt(flatten)]
    rotate_network_key: RotateNetworkKey,
}

impl RotateFullNodeNetworkKey {
    pub fn execute(self) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        self.rotate_network_key.execute(FULLNODE_NETWORK_KEY)
    }
}

fn decode_validator_addresses(
    owner_account: &AccountAddress,
    shared_val_netaddr_keys: &HashMap<netaddr::KeyVersion, netaddr::Key>,
    validator_addresses: &[RawEncNetworkAddress],
) -> Result<Vec<NetworkAddress>, Error> {
    decrypt_addresses(owner_account, shared_val_netaddr_keys, validator_addresses)
        // extract the base transport protocols only
        .map(|maybe_addr| {
            maybe_addr
                .map_err(|err| {
                    Error::UnexpectedError(format!(
                        "Unable to read validator network address: {}",
                        err
                    ))
                })
                .and_then(|addr| parse_transport(&addr))
        })
        .collect::<Result<Vec<_>, _>>()
}

fn decode_full_node_addresses(
    owner_account: &AccountAddress,
    full_node_addresses: &[RawNetworkAddress],
) -> Result<Vec<NetworkAddress>, Error> {
    deserialize_addresses(owner_account, full_node_addresses)
        // extract the base transport protocols only
        .map(|maybe_addr| {
            maybe_addr
                .map_err(|err| {
                    Error::UnexpectedError(format!(
                        "Unable to read full node network address: {}",
                        err
                    ))
                })
                .and_then(|addr| parse_transport(&addr))
        })
        .collect::<Result<Vec<_>, _>>()
}

// TODO(philiphayes): HACK: add TransportProtocol type?

fn parse_transport(addr: &NetworkAddress) -> Result<NetworkAddress, Error> {
    let protocols: Vec<_> = addr
        .clone()
        .into_iter()
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
        .collect();
    NetworkAddress::try_from(protocols).map_err(|_| {
        Error::UnexpectedError(format!(
            "Error: network address does not contain any transport protocols: '{}'",
            addr
        ))
    })
}
