// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

pub mod crypto;
pub mod error;
mod interface;
pub mod transaction;

use crate::interface::{
    LIBRA_ADDRESS_SIZE, LIBRA_AUTHKEY_SIZE, LIBRA_PRIVKEY_SIZE, LIBRA_PUBKEY_SIZE,
};
use libra_crypto::ed25519::{ED25519_PRIVATE_KEY_LENGTH, ED25519_PUBLIC_KEY_LENGTH};
use libra_types::{account_address::AccountAddress, transaction::authenticator::AuthenticationKey};

static_assertions::const_assert_eq!(LIBRA_PUBKEY_SIZE, ED25519_PUBLIC_KEY_LENGTH as u32);
static_assertions::const_assert_eq!(LIBRA_PRIVKEY_SIZE, ED25519_PRIVATE_KEY_LENGTH as u32);
static_assertions::const_assert_eq!(LIBRA_AUTHKEY_SIZE, AuthenticationKey::LENGTH as u32);
static_assertions::const_assert_eq!(LIBRA_ADDRESS_SIZE, AccountAddress::LENGTH as u32);
