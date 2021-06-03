// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{Config, ConfigPath},
    error::Error,
    secure_backend::ValidatorBackend,
    storage::to_x25519,
    transaction::build_raw_transaction,
};
use core::str::FromStr;
use diem_config::config::HANDSHAKE_VERSION;
use diem_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_ACCOUNT,
    VALIDATOR_NETWORK_KEY,
};
use diem_network_address_encryption::Encryptor;
use diem_secure_storage::{CryptoStorage, KVStorage, Storage};
use diem_transaction_builder::stdlib as transaction_builder;
use diem_types::{
    account_address::AccountAddress,
    chain_id::ChainId,
    network_address::{NetworkAddress, Protocol},
    transaction::{SignedTransaction, Transaction},
};
use std::net::{Ipv4Addr, ToSocketAddrs};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(flatten)]
    pub config: ConfigPath,
    #[structopt(long, required_unless("config"))]
    pub chain_id: Option<ChainId>,
    #[structopt(flatten)]
    pub validator_backend: ValidatorBackend,
}

impl ValidatorConfig {
    pub fn config(&self) -> Result<Config, Error> {
        self.config
            .load()?
            .override_chain_id(self.chain_id)
            .override_validator_backend(&self.validator_backend.validator_backend)
    }

    pub fn build_transaction(
        &self,
        sequence_number: u64,
        fullnode_address: NetworkAddress,
        validator_address: NetworkAddress,
        reconfigure: bool,
        disable_address_validation: bool,
    ) -> Result<Transaction, Error> {
        let config = self.config()?;
        let storage = Storage::from(&config.validator_backend);
        let chain_id = config.chain_id;

        build_validator_config_transaction(
            storage,
            chain_id,
            sequence_number,
            fullnode_address,
            validator_address,
            reconfigure,
            disable_address_validation,
        )
        .map_err(|e| {
            Error::UnexpectedError(format!(
                "Error building validator config transaction: {}",
                e
            ))
        })
    }
}

/// Requires that the validator storage has the following keys set:
/// * OWNER_ACCOUNT
/// * CONSENSUS_KEY
/// * FULLNODE_NETWORK_KEY
/// * VALIDATOR_NETWORK_KEY
/// * VALIDATOR_NETWORK_ADDRESS_KEYS - For encrypting network addresses
/// * OPERATOR_ACCOUNT
/// * OPERATOR_KEY
pub fn build_validator_config_transaction<S: KVStorage + CryptoStorage>(
    mut validator_storage: S,
    chain_id: ChainId,
    sequence_number: u64,
    fullnode_address: NetworkAddress,
    validator_address: NetworkAddress,
    reconfigure: bool,
    disable_address_validation: bool,
) -> anyhow::Result<Transaction> {
    if !disable_address_validation {
        // Verify addresses
        validate_address("validator address", &validator_address)?;
        validate_address("fullnode address", &fullnode_address)?;
    }

    let owner_account = validator_storage
        .get::<AccountAddress>(OWNER_ACCOUNT)
        .map(|v| v.value)?;
    let operator_account = validator_storage
        .get::<AccountAddress>(OPERATOR_ACCOUNT)
        .map(|v| v.value)?;

    let consensus_key = validator_storage
        .get_public_key(CONSENSUS_KEY)
        .map(|v| v.public_key)?;
    let fullnode_network_key = validator_storage
        .get_public_key(FULLNODE_NETWORK_KEY)
        .map(|v| v.public_key)
        .map_err(|e| Error::UnexpectedError(e.to_string()))
        .and_then(to_x25519)?;
    let validator_network_key = validator_storage
        .get_public_key(VALIDATOR_NETWORK_KEY)
        .map(|v| v.public_key)
        .map_err(|e| Error::UnexpectedError(e.to_string()))
        .and_then(to_x25519)?;

    // Build Validator address including protocols and encryption
    // Append ln-noise-ik and ln-handshake protocols to base network addresses
    // and encrypt the validator address.
    let validator_address =
        validator_address.append_prod_protos(validator_network_key, HANDSHAKE_VERSION);
    let encryptor = Encryptor::new(&mut validator_storage);
    let validator_addresses = encryptor
        .encrypt(
            &[validator_address],
            owner_account,
            sequence_number + if reconfigure { 1 } else { 0 },
        )
        .map_err(|e| {
            Error::UnexpectedError(format!("Error encrypting validator address: {}", e))
        })?;

    // Build Fullnode address including protocols
    let fullnode_address =
        fullnode_address.append_prod_protos(fullnode_network_key, HANDSHAKE_VERSION);

    // Generate the validator config script
    let transaction_callback = if reconfigure {
        transaction_builder::encode_set_validator_config_and_reconfigure_script_function
    } else {
        transaction_builder::encode_register_validator_config_script_function
    };
    let validator_config_script = transaction_callback(
        owner_account,
        consensus_key.to_bytes().to_vec(),
        validator_addresses,
        bcs::to_bytes(&vec![fullnode_address]).unwrap(),
    )
    .into_script_function();

    // Create and sign the validator-config transaction
    let raw_txn = build_raw_transaction(
        chain_id,
        operator_account,
        sequence_number,
        validator_config_script,
    );
    let public_key = validator_storage
        .get_public_key(OPERATOR_KEY)
        .map(|v| v.public_key)?;
    let signature = validator_storage.sign(OPERATOR_KEY, &raw_txn)?;
    let signed_txn = SignedTransaction::new(raw_txn, public_key, signature);
    let txn = Transaction::UserTransaction(signed_txn);

    Ok(txn)
}

/// Validates an address to have a DNS/IP and a port, as well as to be resolvable
pub fn validate_address(
    address_name: &'static str,
    network_address: &NetworkAddress,
) -> Result<(), Error> {
    let mut has_addr = false;
    let mut has_port = false;
    // Only allow DNS and IP addresses
    for protocol in network_address.as_slice().iter() {
        match protocol {
            Protocol::Ip4(_) => has_addr = true,
            Protocol::Dns4(dns_name) => {
                let dns_name = format!("{}", dns_name);
                if Ipv4Addr::from_str(&dns_name).is_ok() {
                    return Err(Error::CommandArgumentError(format!(
                        "{}: Please use the /ip4/ protocol for IP addresses",
                        address_name
                    )));
                }
                has_addr = true
            }
            Protocol::Tcp(_) => has_port = true,
            Protocol::Dns(_) | Protocol::Ip6(_) | Protocol::Dns6(_) => {
                return Err(Error::CommandArgumentError(format!(
                    "{}: IPv6 is currently not supported.  Protocol: '{}'",
                    address_name, protocol
                )))
            }
            _ => {
                return Err(Error::CommandArgumentError(format!(
                    "{}: Invalid protocol '{}'",
                    address_name, protocol
                )))
            }
        }
    }

    if !has_addr || !has_port {
        return Err(Error::CommandArgumentError(format!(
            "{}: Address must have a scheme (e.g. /ipv4/) and a port (e.g. /tcp/)",
            address_name
        )));
    }

    // Ensure that the address resolves to IP addresses
    let addrs = network_address.to_socket_addrs().map_err(|err| {
        Error::CommandArgumentError(format!(
            "{}: Failed to resolve address '{}': {}",
            address_name, network_address, err
        ))
    })?;

    if addrs.len() < 1 {
        return Err(Error::CommandArgumentError(format!(
            "{}: Resolved to no IP addresses '{}'",
            address_name, network_address
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_inputs() {
        let no_port = NetworkAddress::from_str("/ip4/127.0.0.1").unwrap();
        let no_ip = NetworkAddress::from_str("/tcp/1234").unwrap();
        let ipv6 = NetworkAddress::from_str("/dns6/localhost").unwrap();
        let ipv4_and_ipv6 = NetworkAddress::from_str("/dns/localhost").unwrap();
        let bad_protocol = NetworkAddress::from_str("/ln-handshake/0").unwrap();
        let ip_in_dns = NetworkAddress::from_str("/dns4/127.0.0.1/tcp/1234").unwrap();

        validate_address("no_port", &no_port).expect_err("Failed to check for port");
        validate_address("no_ip", &no_ip).expect_err("Failed to check for no IP");
        validate_address("ipv6", &ipv6).expect_err("Failed to check for ipv6");
        validate_address("ipv4_and_ipv6", &ipv4_and_ipv6).expect_err("Failed to check for ipv6");
        validate_address("bad_protocol", &bad_protocol)
            .expect_err("Failed to check for bad protocol");
        validate_address("ip_in_dns", &ip_in_dns).expect_err("Failed to check for ip in DNS");
    }

    #[test]
    fn test_valid_inputs() {
        let ip = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
        let dns = NetworkAddress::from_str("/dns4/localhost/tcp/1234").unwrap();

        validate_address("ip", &ip).expect("IP failed to validate");
        validate_address("dns", &dns).expect("DNS failed to validate");
    }
}
