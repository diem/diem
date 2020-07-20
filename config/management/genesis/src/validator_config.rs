// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::HANDSHAKE_VERSION;
use libra_global_constants::OWNER_ACCOUNT;
use libra_management::{
    constants,
    error::Error,
    secure_backend::{SharedBackend, ValidatorBackend},
    storage::StorageWrapper,
    validator_config::{
        create_transaction, create_validator_config_script, encode_address,
        encode_validator_address, fetch_keys_from_storage, owner_from_key,
    },
};
use libra_network_address::{
    encrypted::{TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION},
    NetworkAddress,
};
use libra_secure_storage::Value;
use libra_types::{chain_id::ChainId, transaction::Transaction};
use structopt::StructOpt;

// TODO(davidiw) add operator_address, since that will eventually be the identity producing this.
#[derive(Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long)]
    owner_name: String,
    #[structopt(long)]
    validator_address: NetworkAddress,
    #[structopt(long)]
    fullnode_address: NetworkAddress,
    #[structopt(long)]
    chain_id: ChainId,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    shared_backend: SharedBackend,
}

impl ValidatorConfig {
    pub fn execute(self) -> Result<Transaction, Error> {
        let storage_name = self.validator_backend.name();
        let sequence_number = 0u64;
        // Only supports one address for now
        let idx_addr = 0u32;

        // Fetch the owner key from remote storage using the owner_name and derive an address
        let owner_account = owner_from_key(
            storage_name,
            &self.shared_backend.shared_backend,
            self.owner_name,
        )?;

        // Fetch keys from storage
        let (consensus_key, fullnode_network_key, validator_network_key) =
            fetch_keys_from_storage(storage_name, &self.validator_backend.validator_backend)?;

        // Append ln-noise-ik and ln-handshake protocols to base network addresses
        // and encrypt the validator address.
        let validator_address = self
            .validator_address
            .append_prod_protos(validator_network_key, HANDSHAKE_VERSION);
        let raw_enc_validator_address = encode_validator_address(
            validator_address,
            &owner_account,
            sequence_number,
            &TEST_SHARED_VAL_NETADDR_KEY,
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            idx_addr,
        )?;

        let fullnode_address = self
            .fullnode_address
            .append_prod_protos(fullnode_network_key, HANDSHAKE_VERSION);
        let raw_fullnode_address = encode_address(fullnode_address)?;

        let validator_config = libra_types::validator_config::ValidatorConfig {
            consensus_public_key: consensus_key,
            validator_network_identity_public_key: validator_network_key,
            validator_network_address: raw_enc_validator_address,
            full_node_network_identity_public_key: fullnode_network_key,
            full_node_network_address: raw_fullnode_address,
        };

        // Create the validator config script for the validator node
        let validator_config_script =
            create_validator_config_script(owner_account, validator_config);

        // Create and sign the validator-config transaction
        let validator_config_tx = create_transaction(
            storage_name,
            &self.validator_backend.validator_backend,
            sequence_number,
            self.chain_id,
            "validator-config",
            validator_config_script,
        )?;
        let validator_config_tx = Transaction::UserTransaction(validator_config_tx);

        // Write validator config to local storage to save for verification later on
        let mut validator_storage =
            StorageWrapper::new(storage_name, &self.validator_backend.validator_backend)?;
        validator_storage.set(
            constants::VALIDATOR_CONFIG,
            Value::Transaction(validator_config_tx.clone()),
        )?;

        // Save the owner account in local storage for deployment later on
        validator_storage.set(OWNER_ACCOUNT, Value::String(owner_account.to_string()))?;

        // Upload the validator config to shared storage
        let storage_name = self.shared_backend.name();
        let mut shared_storage =
            StorageWrapper::new(storage_name, &self.shared_backend.shared_backend)?;
        shared_storage.set(
            constants::VALIDATOR_CONFIG,
            Value::Transaction(validator_config_tx.clone()),
        )?;

        Ok(validator_config_tx)
    }
}
