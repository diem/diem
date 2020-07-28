// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{constants, error::Error, secure_backend::ValidatorBackend, storage::StorageWrapper};
use libra_config::config::HANDSHAKE_VERSION;
use libra_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_ACCOUNT,
    VALIDATOR_NETWORK_KEY,
};
use libra_network_address::{
    encrypted::{
        encrypt_addresses, TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION,
    },
    serialize_addresses, NetworkAddress,
};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    chain_id::ChainId,
    transaction::{RawTransaction, Transaction},
};
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long)]
    pub chain_id: ChainId,
    #[structopt(flatten)]
    pub validator_backend: ValidatorBackend,
}

impl ValidatorConfig {
    pub fn build_transaction(
        &self,
        sequence_number: u64,
        full_node_base_addresses: Vec<NetworkAddress>,
        validator_base_addresses: Vec<NetworkAddress>,
        reconfigure: bool,
    ) -> Result<Transaction, Error> {
        let mut storage = StorageWrapper::new(
            self.validator_backend.name(),
            &self.validator_backend.validator_backend,
        )?;
        let owner_account = storage.account_address(OWNER_ACCOUNT)?;

        let consensus_key = storage.ed25519_public_from_private(CONSENSUS_KEY)?;
        let full_node_network_key = storage.x25519_public_from_private(FULLNODE_NETWORK_KEY)?;
        let validator_network_key = storage.x25519_public_from_private(VALIDATOR_NETWORK_KEY)?;

        // Build Validator addresses including protocols and encryption
        // Append ln-noise-ik and ln-handshake protocols to base network addresses
        // and encrypt the validator address.
        // We also dedup the base addresses
        let validator_addresses: Vec<_> = dedup_addresses(validator_base_addresses)
            .into_iter()
            .map(|addr| addr.append_prod_protos(validator_network_key, HANDSHAKE_VERSION))
            .collect();
        let full_node_addresses: Vec<_> = dedup_addresses(full_node_base_addresses)
            .into_iter()
            .map(|addr| addr.append_prod_protos(full_node_network_key, HANDSHAKE_VERSION))
            .collect();

        let raw_enc_validator_addresses = encrypt_addresses(
            &owner_account,
            &TEST_SHARED_VAL_NETADDR_KEY,
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            // This needs to be distinct, genesis = 0, post genesis sequence_number + 1
            sequence_number + if reconfigure { 1 } else { 0 },
            &validator_addresses,
        )
        .map(|maybe_raw_enc_addr| {
            maybe_raw_enc_addr.map(Into::<Vec<_>>::into).map_err(|err| {
                Error::UnexpectedError(format!("error encrypting validator address: {}", err))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

        let raw_fullnode_addresses = serialize_addresses(&full_node_addresses)
            .map(|maybe_raw_enc_addr| {
                maybe_raw_enc_addr.map(Into::<Vec<_>>::into).map_err(|err| {
                    Error::UnexpectedError(format!("error serializing fullnode address: {}", err))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Generate the validator config script
        let transaction_callback = if reconfigure {
            transaction_builder::encode_set_validator_config_and_reconfigure_script
        } else {
            transaction_builder::encode_set_validator_config_script
        };
        let validator_config_script = transaction_callback(
            owner_account,
            consensus_key.to_bytes().to_vec(),
            raw_enc_validator_addresses,
            raw_fullnode_addresses,
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
            self.chain_id,
        );
        let signed_txn = storage.sign(OPERATOR_KEY, "validator-config", raw_txn)?;
        let txn = Transaction::UserTransaction(signed_txn);

        Ok(txn)
    }
}

// Dedup a vec of addresses while preserving the relative ordering
fn dedup_addresses(addrs: Vec<NetworkAddress>) -> Vec<NetworkAddress> {
    let mut deduped_addrs = Vec::new();
    for addr in addrs {
        if !deduped_addrs.contains(&addr) {
            deduped_addrs.push(addr);
        }
    }
    deduped_addrs
}
