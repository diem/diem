// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod proto;
pub use runtime::StateSyncRuntime;

mod runtime;
mod state_sync_service;
