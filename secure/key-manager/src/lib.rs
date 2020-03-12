// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod key_manager;

pub use crate::key_manager::KeyManager;

#[cfg(test)]
mod tests;
