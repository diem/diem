// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{constants, error::Error, SecureBackends};
use libra_crypto::{hash::CryptoHash, x25519, ValidCryptoMaterial};
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

        let consensus_key = local
            .get_public_key(constants::CONSENSUS_KEY)
            .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
            .public_key;
        let fullnode_network_key = local
            .get_public_key(constants::FULLNODE_NETWORK_KEY)
            .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
            .public_key;
        let validator_network_key = local
            .get_public_key(constants::VALIDATOR_NETWORK_KEY)
            .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
            .public_key;

        let fullnode_address = RawNetworkAddress::try_from(&self.fullnode_address)
            .expect("Unable to serialize fullnode network address from config");
        let validator_address = RawNetworkAddress::try_from(&self.validator_address)
            .expect("Unable to serialize validator network address from config");

        let fullnode_network_key = fullnode_network_key.to_bytes();
        let fullnode_network_key: x25519::PublicKey = fullnode_network_key
            .as_ref()
            .try_into()
            .expect("Unable to decode x25519 from fullnode_network_key");

        let validator_network_key = validator_network_key.to_bytes();
        let validator_network_key: x25519::PublicKey = validator_network_key
            .as_ref()
            .try_into()
            .expect("Unable to decode x25519 from validator_network_key");

        let owner_key = local
            .get_public_key(constants::OWNER_KEY)
            .map_err(|e| Error::LocalStorageReadError(e.to_string()))?
            .public_key;

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
        let expiration_time = RealTimeService::new().now() + constants::TXN_EXPIRATION_SECS;
        let raw_transaction = RawTransaction::new_script(
            sender,
            sequence_number,
            script,
            constants::MAX_GAS_AMOUNT,
            constants::GAS_UNIT_PRICE,
            Duration::from_secs(expiration_time),
        );
        let signature = local
            .sign_message(constants::OWNER_KEY, &raw_transaction.hash())
            .map_err(|e| Error::LocalStorageSigningError(e.to_string()))?;
        let signed_txn = SignedTransaction::new(raw_transaction, owner_key, signature);
        let txn = Transaction::UserTransaction(signed_txn);

        if let Some(remote) = self.backends.remote {
            let mut remote: Box<dyn Storage> = remote.try_into()?;
            if !remote.available() {
                return Err(Error::RemoteStorageUnavailable);
            }

            let txn = Value::Transaction(txn.clone());
            remote
                .create_with_default_policy(constants::VALIDATOR_CONFIG, txn)
                .map_err(|e| Error::RemoteStorageWriteError(e.to_string()))?;
        }

        Ok(txn)
    }
}
