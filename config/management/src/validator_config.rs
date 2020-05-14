// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, SecureBackends};
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
        if !local.available() {
            return Err(Error::LocalStorageUnavailable);
        }

        // Step 1) Retrieve keys from local storage
        let consensus_key = ed25519_from_storage(CONSENSUS_KEY, local.as_mut())?;
        let fullnode_network_key = x25519_from_storage(FULLNODE_NETWORK_KEY, local.as_mut())?;
        let validator_network_key = x25519_from_storage(VALIDATOR_NETWORK_KEY, local.as_mut())?;
        let owner_key = ed25519_from_storage(OWNER_KEY, local.as_mut())?;

        let fullnode_address = RawNetworkAddress::try_from(&self.fullnode_address)
            .map_err(|e| Error::UnexpectedError(format!("(fullnode_address) {}", e)))?;
        let validator_address = RawNetworkAddress::try_from(&self.validator_address)
            .map_err(|e| Error::UnexpectedError(format!("(validator_address) {}", e)))?;

        // Step 2) Generate transaction

        // TODO(davidiw): The signing key, parameter 2, will be deleted soon, so this is a
        // temporary hack to reduce over-engineering.
        let script = transaction_builder::encode_register_validator_script(
            consensus_key.to_bytes().to_vec(),
            owner_key.to_bytes().to_vec(),
            validator_network_key.to_bytes(),
            validator_address.into(),
            fullnode_network_key.to_bytes(),
            fullnode_address.into(),
        );

        let sender = self.owner_address;
        // TODO(davidiw): In genesis this is irrelevant -- afterward we need to obtain the
        // current sequence number by querying the blockchain.
        let sequence_number = 0;
        let expiration_time = RealTimeService::new().now() + crate::constants::TXN_EXPIRATION_SECS;
        let raw_transaction = RawTransaction::new_script(
            sender,
            sequence_number,
            script,
            crate::constants::MAX_GAS_AMOUNT,
            crate::constants::GAS_UNIT_PRICE,
            crate::constants::GAS_CURRENCY_CODE.to_owned(),
            Duration::from_secs(expiration_time),
        );
        let signature = local
            .sign_message(OWNER_KEY, &raw_transaction.hash())
            .map_err(|e| Error::LocalStorageSigningError(e.to_string()))?;
        let signed_txn = SignedTransaction::new(raw_transaction, owner_key, signature);
        let txn = Transaction::UserTransaction(signed_txn);

        // Step 3) Submit to remote storage

        if let Some(remote) = self.backends.remote {
            let mut remote: Box<dyn Storage> = remote.try_into()?;
            if !remote.available() {
                return Err(Error::RemoteStorageUnavailable);
            }

            let txn = Value::Transaction(txn.clone());
            remote
                .set(crate::constants::VALIDATOR_CONFIG, txn)
                .map_err(|e| Error::RemoteStorageWriteError(e.to_string()))?;
        }

        Ok(txn)
    }
}

fn ed25519_from_storage(
    key_name: &str,
    storage: &mut dyn Storage,
) -> Result<Ed25519PublicKey, Error> {
    Ok(storage
        .get_public_key(key_name)
        .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
        .public_key)
}

fn x25519_from_storage(
    key_name: &str,
    storage: &mut dyn Storage,
) -> Result<x25519::PublicKey, Error> {
    let edkey = storage
        .export_private_key(key_name)
        .map_err(|e| Error::LocalStorageReadError(e.to_string()))?;
    let xkey: Result<x25519::PrivateKey, _> = edkey.to_bytes().as_ref().try_into();
    xkey.map(|k| k.public_key())
        .map_err(|e| Error::UnexpectedError(format!("({}) {}", key_name, e)))
}
