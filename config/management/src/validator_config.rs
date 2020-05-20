// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{constants, error::Error, SecureBackends};
use libra_crypto::{ed25519::Ed25519PublicKey, hash::CryptoHash, x25519, ValidCryptoMaterial};
use libra_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, OWNER_KEY, VALIDATOR_NETWORK_KEY,
};
use libra_network_address::{NetworkAddress, RawNetworkAddress};
use libra_secure_storage::{Storage, Value};
use libra_secure_time::{RealTimeService, TimeService};
use libra_types::{
    account_address::AccountAddress,
    transaction::{RawTransaction, SignedTransaction, Transaction},
};
use std::{
    convert::{TryFrom, TryInto},
    time::Duration,
};
use structopt::StructOpt;

// TODO(davidiw) add operator_address, since that will eventually be the identity producing this.
#[derive(Debug, StructOpt)]
pub struct ValidatorConfig {
    #[structopt(long)]
    owner_address: AccountAddress,
    #[structopt(long)]
    validator_address: NetworkAddress,
    #[structopt(long)]
    fullnode_address: NetworkAddress,
    #[structopt(flatten)]
    backends: SecureBackends,
}

impl ValidatorConfig {
    pub fn execute(self) -> Result<Transaction, Error> {
        let mut local: Box<dyn Storage> = self.backends.local.try_into()?;
        local
            .available()
            .map_err(|e| Error::LocalStorageUnavailable(e.to_string()))?;

        // Step 1) Retrieve keys from local storage
        let consensus_key = ed25519_from_storage(CONSENSUS_KEY, local.as_mut())?;
        let fullnode_network_key = x25519_from_storage(FULLNODE_NETWORK_KEY, local.as_mut())?;
        let validator_network_key = x25519_from_storage(VALIDATOR_NETWORK_KEY, local.as_mut())?;
        let owner_key = ed25519_from_storage(OWNER_KEY, local.as_mut())?;

        // append ln-noise-ik and ln-handshake protocols to base network addresses

        let validator_address = self
            .validator_address
            .clone()
            .append_prod_protos(validator_network_key.clone(), constants::HANDSHAKE_VERSION);
        let raw_validator_address = RawNetworkAddress::try_from(&validator_address)
            .map_err(|e| Error::UnexpectedError(format!("(raw_validator_address) {}", e)))?;

        let fullnode_address = self
            .fullnode_address
            .clone()
            .append_prod_protos(fullnode_network_key.clone(), constants::HANDSHAKE_VERSION);
        let raw_fullnode_address = RawNetworkAddress::try_from(&fullnode_address)
            .map_err(|e| Error::UnexpectedError(format!("(raw_fullnode_address) {}", e)))?;

        // Step 2) Generate transaction

        // TODO(davidiw): The signing key, parameter 2, will be deleted soon, so this is a
        // temporary hack to reduce over-engineering.
        // TODO(philiphayes): remove network identity pubkey field from struct when
        // transition complete
        let script = transaction_builder::encode_register_validator_script(
            consensus_key.to_bytes().to_vec(),
            owner_key.to_bytes().to_vec(),
            validator_network_key.to_bytes(),
            raw_validator_address.into(),
            fullnode_network_key.to_bytes(),
            raw_fullnode_address.into(),
        );

        let sender = self.owner_address;
        // TODO(davidiw): In genesis this is irrelevant -- afterward we need to obtain the
        // current sequence number by querying the blockchain.
        let sequence_number = 0;
        let expiration_time = RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS;
        let raw_transaction = RawTransaction::new_script(
            sender,
            sequence_number,
            script,
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            constants::GAS_CURRENCY_CODE.to_owned(),
            Duration::from_secs(expiration_time),
        );
        let signature = local
            .sign_message(OWNER_KEY, &raw_transaction.hash())
            .map_err(|e| {
                Error::LocalStorageSigningError("validator-config", OWNER_KEY, e.to_string())
            })?;
        let signed_txn = SignedTransaction::new(raw_transaction, owner_key, signature);
        let txn = Transaction::UserTransaction(signed_txn);

        // Step 3) Submit to remote storage

        if let Some(remote) = self.backends.remote {
            let mut remote: Box<dyn Storage> = remote.try_into()?;
            remote
                .available()
                .map_err(|e| Error::RemoteStorageUnavailable(e.to_string()))?;
            let txn = Value::Transaction(txn.clone());
            remote.set(constants::VALIDATOR_CONFIG, txn).map_err(|e| {
                Error::RemoteStorageWriteError(constants::VALIDATOR_CONFIG, e.to_string())
            })?;
        }

        Ok(txn)
    }
}

fn ed25519_from_storage(
    key_name: &'static str,
    storage: &mut dyn Storage,
) -> Result<Ed25519PublicKey, Error> {
    Ok(storage
        .get_public_key(key_name)
        .map_err(|e| Error::LocalStorageReadError(key_name, e.to_string()))?
        .public_key)
}

fn x25519_from_storage(
    key_name: &'static str,
    storage: &mut dyn Storage,
) -> Result<x25519::PublicKey, Error> {
    let edkey = ed25519_from_storage(key_name, storage)?;
    x25519::PublicKey::from_ed25519_public_bytes(&edkey.to_bytes())
        .map_err(|e| Error::UnexpectedError(e.to_string()))
}
