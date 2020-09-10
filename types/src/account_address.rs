// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::transaction::authenticator::AuthenticationKey;
use libra_crypto::ed25519::Ed25519PublicKey;

pub use move_core_types::account_address::AccountAddress;

pub fn from_public_key(public_key: &Ed25519PublicKey) -> AccountAddress {
    AuthenticationKey::ed25519(public_key).derived_address()
}
