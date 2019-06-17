// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
//! A secret service providing cryptographic operations on secret keys, will be used in future
//! releases.
pub mod crypto_wrappers;
pub mod proto;
pub mod secret_service_client;

pub mod secret_service_node;
pub mod secret_service_server;
