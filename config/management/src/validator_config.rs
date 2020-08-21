// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{Config, ConfigPath},
    error::Error,
    secure_backend::ValidatorBackend,
    transaction::build_raw_transaction,
};
use core::str::FromStr;
use libra_config::config::HANDSHAKE_VERSION;
use libra_crypto::ValidCryptoMaterial;
use libra_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_ACCOUNT,
    VALIDATOR_NETWORK_KEY,
};
use libra_network_address::{
    encrypted::{
        RawEncNetworkAddress, TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION,
    },
    NetworkAddress, Protocol, RawNetworkAddress,
};
use libra_types::{chain_id::ChainId, transaction::Transaction};
use std::{
    convert::TryFrom,
    net::{Ipv4Addr, ToSocketAddrs},
};
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
    ) -> Result<Transaction, Error> {
        let config = self.config()?;
        let mut storage = config.validator_backend();

        let owner_account = storage.account_address(OWNER_ACCOUNT)?;

        let consensus_key = storage.ed25519_public_from_private(CONSENSUS_KEY)?;
        let fullnode_network_key = storage.x25519_public_from_private(FULLNODE_NETWORK_KEY)?;
        let validator_network_key = storage.x25519_public_from_private(VALIDATOR_NETWORK_KEY)?;

        // Build Validator address including protocols and encryption
        // Append ln-noise-ik and ln-handshake protocols to base network addresses
        // and encrypt the validator address.
        let validator_address =
            validator_address.append_prod_protos(validator_network_key, HANDSHAKE_VERSION);
        let raw_validator_address = encode_address(validator_address)?;
        // Only supports one address for now
        let key = TEST_SHARED_VAL_NETADDR_KEY;
        let version = TEST_SHARED_VAL_NETADDR_KEY_VERSION;
        let enc_validator_address = raw_validator_address.encrypt(
            &key,
            version,
            &owner_account,
            // This needs to be distinct, genesis = 0, post genesis sequence_number + 1
            sequence_number + if reconfigure { 1 } else { 0 },
            0, // addr_idx
        );
        let raw_enc_validator_address = RawEncNetworkAddress::try_from(&enc_validator_address)
            .map_err(|e| {
                Error::UnexpectedError(format!(
                    "error serializing encrypted address: '{:?}', error: {}",
                    enc_validator_address, e
                ))
            })?;

        // Build Fullnode address including protocols
        let fullnode_address =
            fullnode_address.append_prod_protos(fullnode_network_key, HANDSHAKE_VERSION);
        let raw_fullnode_address = encode_address(fullnode_address)?;

        // Generate the validator config script
        // TODO(philiphayes): remove network identity pubkey field from struct
        let transaction_callback = if reconfigure {
            transaction_builder::encode_set_validator_config_and_reconfigure_script
        } else {
            transaction_builder::encode_register_validator_config_script
        };
        let validator_config_script = transaction_callback(
            owner_account,
            consensus_key.to_bytes().to_vec(),
            validator_network_key.to_bytes(),
            raw_enc_validator_address.into(),
            fullnode_network_key.to_bytes(),
            raw_fullnode_address.into(),
        );

        // Create and sign the validator-config transaction
        let raw_txn = build_raw_transaction(
            config.chain_id,
            storage.account_address(OPERATOR_ACCOUNT)?,
            sequence_number,
            validator_config_script,
        );
        let signed_txn = storage.sign(OPERATOR_KEY, "validator-config", raw_txn)?;
        let txn = Transaction::UserTransaction(signed_txn);

        Ok(txn)
    }
}

/// Encode an address into bytes
fn encode_address(address: NetworkAddress) -> Result<RawNetworkAddress, Error> {
    RawNetworkAddress::try_from(&address).map_err(|e| {
        Error::UnexpectedError(format!(
            "error serializing address: '{}', error: {}",
            address, e
        ))
    })
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
