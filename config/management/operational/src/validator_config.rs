// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::config::HANDSHAKE_VERSION;
use libra_crypto::x25519;
use libra_global_constants::VALIDATOR_NETWORK_KEY;
use libra_management::{
    error::Error,
    json_rpc::JsonRpcClientWrapper,
    secure_backend::{SharedBackend, ValidatorBackend},
    storage::StorageWrapper,
    validator_config::{
        decode_validator_address, encode_address, encode_validator_address,
        update_config_and_reconfigure,
    },
    TransactionContext,
};
use libra_network_address::{
    encrypted::{TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION},
    NetworkAddress,
};
use libra_types::{account_address::AccountAddress, validator_config::ValidatorConfig};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct GetValidatorConfig {
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080")]
    host: String,
    #[structopt(long, help = "Owner account address")]
    account: AccountAddress,
}

impl GetValidatorConfig {
    pub fn execute(self) -> Result<ValidatorConfig, Error> {
        let client = JsonRpcClientWrapper::new(self.host);
        client.validator_config(self.account)
    }
}

// TODO: Load all chain IDs from the host
#[derive(Debug, StructOpt)]
pub struct SetValidatorConfig {
    #[structopt(long, help = "Chain Id representing the chain (one byte)")]
    chain_id: u8,
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080")]
    host: String,
    #[structopt(long, help = "Owner account address")]
    owner_account: AccountAddress,
    #[structopt(long, help = "Validator Network Address")]
    validator_address: Option<NetworkAddress>,
    #[structopt(long, help = "Full Node Network Address")]
    fullnode_address: Option<NetworkAddress>,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
    #[structopt(flatten)]
    shared_backend: SharedBackend,
}

impl SetValidatorConfig {
    pub fn execute(self) -> Result<TransactionContext, Error> {
        let idx_addr = 0;
        let storage_name = self.validator_backend.name();

        // Get current sequence number
        let client = JsonRpcClientWrapper::new(self.host);
        let sequence_number = client.sequence_number(self.owner_account)?;

        // Update validator or full node address accordingly
        let mut validator_config = client.validator_config(self.owner_account)?;
        if let Some(validator_address) = self.validator_address {
            let validator_address = validator_address.append_prod_protos(
                validator_config.validator_network_identity_public_key,
                HANDSHAKE_VERSION,
            );
            validator_config.validator_network_address = encode_validator_address(
                validator_address,
                &self.owner_account,
                sequence_number,
                &TEST_SHARED_VAL_NETADDR_KEY,
                TEST_SHARED_VAL_NETADDR_KEY_VERSION,
                idx_addr,
            )?
        };

        if let Some(fullnode_address) = self.fullnode_address {
            let fullnode_address = fullnode_address.append_prod_protos(
                validator_config.full_node_network_identity_public_key,
                HANDSHAKE_VERSION,
            );
            validator_config.full_node_network_address = encode_address(fullnode_address)?;
        };

        // Submit the transaction with a reconfigure
        update_config_and_reconfigure(
            storage_name,
            &self.validator_backend.validator_backend,
            &client,
            self.owner_account,
            self.chain_id,
            sequence_number,
            validator_config,
        )
    }
}

#[derive(Debug, StructOpt)]
pub struct RotateValidatorNetworkKey {
    #[structopt(long, help = "Chain Id representing the chain (one byte)")]
    chain_id: u8,
    #[structopt(long, help = "JSON-RPC Endpoint (e.g. http://localhost:8080)")]
    host: String,
    #[structopt(long, help = "Owner account address")]
    owner_account: AccountAddress,
    #[structopt(flatten)]
    validator_backend: ValidatorBackend,
}

impl RotateValidatorNetworkKey {
    pub fn execute(self) -> Result<(TransactionContext, x25519::PublicKey), Error> {
        let idx_addr = 0;
        let storage_name = self.validator_backend.name();

        // Get current sequence number
        let client = JsonRpcClientWrapper::new(self.host);

        // TODO: This should be in a single call for both validator config and sequence number
        let sequence_number = client.sequence_number(self.owner_account)?;

        let mut validator_config = client.validator_config(self.owner_account)?;
        let mut validator_address = decode_validator_address(
            validator_config.validator_network_address,
            &self.owner_account,
            &TEST_SHARED_VAL_NETADDR_KEY,
            idx_addr,
        )?;

        let mut storage = StorageWrapper::new(
            self.validator_backend.name(),
            &self.validator_backend.validator_backend,
        )?;

        // Rotate key in storage and network address
        let new_validator_network_key = storage.rotate_key(VALIDATOR_NETWORK_KEY)?;
        let new_validator_network_key = StorageWrapper::x25519(new_validator_network_key)?;
        validator_address.rotate_noise_public_key(
            &validator_config.validator_network_identity_public_key,
            &new_validator_network_key,
        );
        let raw_enc_validator_address = encode_validator_address(
            validator_address,
            &self.owner_account,
            sequence_number,
            &TEST_SHARED_VAL_NETADDR_KEY,
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            idx_addr,
        )?;
        validator_config.validator_network_identity_public_key = new_validator_network_key;
        validator_config.validator_network_address = raw_enc_validator_address;

        // Submit the transaction with a reconfigure
        let transaction_context = update_config_and_reconfigure(
            storage_name,
            &self.validator_backend.validator_backend,
            &client,
            self.owner_account,
            self.chain_id,
            sequence_number,
            validator_config,
        )?;

        Ok((transaction_context, new_validator_network_key))
    }
}
