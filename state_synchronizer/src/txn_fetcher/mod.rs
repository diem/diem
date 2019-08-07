// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This library is used to perform catching up between validators for committed states.
//! Used for node restarts
//!
//! It consists of three components: `Coordinator`, `Downloader` and `Synchronizer`
//!
//! `Downloader` is used to download chunks of transactions from peers
//!
//! `Coordinator` drives synchronization process. It handles new requests and drives whole sync flow
//!
//! `Synchronizer` is an external interface for module.
//! It's used for convenient communication with `Coordinator`.
//! To set it up do: **let synchronizer = StateSynchronizer::setup(network, executor, config)**.
//!
//! It will spawn coordinator and downloader routines and return handle for communication with
//! coordinator.
//! To request synchronization call: **synchronizer.sync_to(peer_id, version).await**
//!
//! Note that it's possible to issue multiple synchronization requests at the same time.
//! `Coordinator` handles it and makes sure each chunk will be downloaded only on

pub use self::coordinator::SyncStatus;

mod coordinator;
mod downloader;
mod synchronizer;

pub use self::synchronizer::{setup_state_synchronizer, StateSynchronizer};
use types::account_address::AccountAddress;

#[cfg(test)]
mod tests;

type PeerId = AccountAddress;
