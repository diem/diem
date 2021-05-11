// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The official Rust SDK for Diem.
//!
//! ## Modules
//!
//! This SDK provides all the necessary components for building on top of the Diem Blockchain. Some of the important modules are:
//!
//! * `client` - Includes a [JSON-RPC client](https://github.com/diem/diem/blob/master/json-rpc/json-rpc-spec.md) implementation
//! * `crypto` - Types used for signing and verifying
//! * `transaction_builder` - Includes helpers for constructing transactions
//! * `types` - Includes types for Diem on-chain data structures
//!
//! ## Example
//!
//! Here is a simple example to show how to create two accounts and do a p2p transfer on testnet:
//!
//! ```no_run
//! use diem_sdk::{
//!     client::{FaucetClient, BlockingClient},
//!     types::{chain_id::ChainId, LocalAccount},
//!     transaction_builder::{Currency, TransactionFactory},
//! };
//! use rand_core::OsRng;
//!
//! let transaction_factory = TransactionFactory::new(ChainId::new(2));
//! let client = BlockingClient::new("http://testnet.diem.com/v1");
//! let faucet = FaucetClient::new("http://testnet.diem.com/mint".to_owned(), "http://testnet.diem.com/v1".to_owned());
//! let mut account_1 = LocalAccount::generate(&mut OsRng);
//! let mut account_2 = LocalAccount::generate(&mut OsRng);
//!
//! // Fund and create account 1 and 2
//! faucet.fund(Currency::XUS.as_str(), account_1.authentication_key(), 100).unwrap();
//! faucet.fund(Currency::XUS.as_str(), account_2.authentication_key(), 50).unwrap();
//!
//! let transaction = account_1.sign_with_transaction_builder(
//!     transaction_factory.peer_to_peer(Currency::XUS, account_2.address(), 10)
//! );
//!
//! client.submit(&transaction).unwrap();
//! client.wait_for_signed_transaction(&transaction, None, None).unwrap();
//!
//! let account_view = client.get_account(account_2.address()).unwrap().into_inner().unwrap();
//! let balance = account_view
//!     .balances
//!     .iter()
//!     .find(|b| b.currency == Currency::XUS)
//!     .unwrap();
//! assert_eq!(balance.amount, 60);
//! ```

#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub mod client {
    pub use diem_client::*;
}

pub mod crypto {
    pub use diem_crypto::*;
}

pub mod transaction_builder;

pub mod types;

pub mod move_types {
    pub use move_core_types::*;
}
