// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

pub mod crypto;
pub mod error;
mod interface;
pub mod transaction;

use crate::interface::{DIEM_ADDRESS_SIZE, DIEM_AUTHKEY_SIZE, DIEM_PRIVKEY_SIZE, DIEM_PUBKEY_SIZE};
use diem_crypto::ed25519::{ED25519_PRIVATE_KEY_LENGTH, ED25519_PUBLIC_KEY_LENGTH};
use diem_types::{account_address::AccountAddress, transaction::authenticator::AuthenticationKey};

static_assertions::const_assert_eq!(DIEM_PUBKEY_SIZE, ED25519_PUBLIC_KEY_LENGTH as u32);
static_assertions::const_assert_eq!(DIEM_PRIVKEY_SIZE, ED25519_PRIVATE_KEY_LENGTH as u32);
static_assertions::const_assert_eq!(DIEM_AUTHKEY_SIZE, AuthenticationKey::LENGTH as u32);
static_assertions::const_assert_eq!(DIEM_ADDRESS_SIZE, AccountAddress::LENGTH as u32);
