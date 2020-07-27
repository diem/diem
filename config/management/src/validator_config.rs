// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::ConfigPath, constants, error::Error, secure_backend::ValidatorBackend};
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
    NetworkAddress, RawNetworkAddress,
};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    chain_id::ChainId,
    transaction::{RawTransaction, Transaction},
};
use std::convert::TryFrom;
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
    pub fn build_transaction(
        &self,
        sequence_number: u64,
        fullnode_address: NetworkAddress,
        validator_address: NetworkAddress,
        reconfigure: bool,
    ) -> Result<Transaction, Error> {
        let config = self
            .config
            .load()?
            .override_chain_id(self.chain_id)
            .override_validator_backend(&self.validator_backend.validator_backend)?;
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
            transaction_builder::encode_set_validator_config_script
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
        let raw_txn = RawTransaction::new_script(
            storage.account_address(OPERATOR_ACCOUNT)?,
            sequence_number,
            validator_config_script,
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS,
            config.chain_id,
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
