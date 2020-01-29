// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Test module.
//!
//! Add new test modules to this list.
//!
//! This is not in a top-level tests directory because each file there gets compiled into a
//! separate binary. The linker ends up repeating a lot of work for each binary to not much
//! benefit.

mod account_universe;
mod create_account;
mod genesis;
mod mint;
mod module_publishing;
mod peer_to_peer;
mod rotate_key;
mod scripts;
mod validator_set_management;
mod verify_txn;
