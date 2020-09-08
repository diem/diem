// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crates provides Faucet service for creating Testnet with simplified on-chain account creation
//! and minting coins.
//!
//! THIS SERVICE SHOULD NEVER BE DEPLOYED TO MAINNET.
//!
//! ## Launch service
//!
//! Launch faucet service local and connect to Testnet:
//!
//! ```bash
//! cargo run -p libra-faucet -- -a 127.0.0.1 -p 8080 -c TESTNET \
//!     -m <treasury-compliance-private-key-path> -s https://testnet.libra.org/v1
//! ```
//!
//! Check help doc for options details:
//!
//! ```bash
//! cargo run -p libra-faucet -- -h
//! ```
//!
//! ## Development
//!
//! Test with libra-swarm by add -m option:
//!
//! ```bash
//! cargo run -p libra-swarm -- -s -l -n 1 -m
//! ```
//!

pub mod mint;
