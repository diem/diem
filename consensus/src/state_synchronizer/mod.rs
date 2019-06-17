// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This library is used to perform synchronization between validators for committed states.
//! This is used for restarts and catching up
//!
//! It consists of three components: `SyncCoordinator`, `Downloader` and `StateSynchronizer`
//!
//! `Downloader` is used to download chunks of transactions from peers
//!
//! `SyncCoordinator` drives synchronization process. It handles new requests from Consensus and
//! drives whole sync flow
//!
//! `StateSynchronizer` is an external interface for module.
//! It's used for convenient communication with `SyncCoordinator`.
//! To set it up do: **let synchronizer = StateSynchronizer::setup(network, executor, config)**.
//!
//! It will spawn coordinator and downloader routines and return handle for communication with
//! coordinator.
//! To request synchronization call: **synchronizer.sync_to(peer_id, version).await**
//!
//! Note that it's possible to issue multiple synchronization requests at the same time.
//! `SyncCoordinator` handles it and makes sure each chunk will be downloaded only once

pub use self::coordinator::SyncStatus;

mod coordinator;
mod downloader;
mod synchronizer;

pub use self::synchronizer::{setup_state_synchronizer, StateSynchronizer};
use types::account_address::AccountAddress;

#[cfg(test)]
mod mocks;
#[cfg(test)]
mod sync_test;

pub type PeerId = AccountAddress;
