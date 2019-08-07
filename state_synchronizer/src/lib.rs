// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![feature(async_await)]
pub mod proto;
pub use runtime::StateSyncRuntime;
pub use txn_fetcher::{setup_state_synchronizer, StateSynchronizer, SyncStatus};

mod counters;
mod runtime;
mod state_sync_service;
mod txn_fetcher;
