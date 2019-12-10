// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Libra Client
//!
//! Client (binary) is the CLI tool to interact with Libra validator.
//! It supposes all public APIs.

pub use libra_crypto::{ed25519::*, test_utils::KeyPair, traits::ValidKeyStringExt};
pub use libra_types::{
    account_address::AccountAddress,
    account_config::association_address,
    transaction::{RawTransaction, TransactionArgument, TransactionPayload},
};
pub use libra_wallet::wallet_library::CryptoHash;
use serde::{Deserialize, Serialize};
pub(crate) mod account_commands;
/// Main instance of client holding corresponding information, e.g. account address.
pub mod client_proxy;
/// Command struct to interact with client.
pub mod commands;
pub(crate) mod dev_commands;
/// gRPC client wrapper to connect to validator.
pub(crate) mod grpc_client;
pub(crate) mod query_commands;
pub(crate) mod transfer_commands;

/// Struct used to store data for each created account.  We track the sequence number
/// so we can create new transactions easily
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
pub struct AccountData {
    /// Address of the account.
    pub address: AccountAddress,
    /// (private_key, public_key) pair if the account is not managed by wallet.
    pub key_pair: Option<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
    /// Latest sequence number maintained by client, it can be different from validator.
    pub sequence_number: u64,
    /// Whether the account is initialized on chain, cached local only, or status unknown.
    pub status: AccountStatus,
}

/// Enum used to represent account status.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AccountStatus {
    /// Account exists only in local cache, it is not persisted on chain.
    Local,
    /// Account is persisted on chain.
    Persisted,
    /// Not able to check account status, probably because client is not able to talk to the
    /// validator.
    Unknown,
}

impl AccountData {
    /// Serialize account keypair if exists.
    pub fn keypair_as_string(&self) -> Option<(String, String)> {
        self.key_pair.as_ref().and_then(|key_pair| {
            let private_key_string = key_pair
                .private_key
                .to_encoded_string()
                .expect("Account private key to convertible to string!");
            let public_key_string = key_pair
                .public_key
                .to_encoded_string()
                .expect("Account public Key not convertible to string!");
            Some((private_key_string, public_key_string))
        })
    }
}
