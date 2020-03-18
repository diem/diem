// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

pub mod account;
pub mod account_resource;
mod data;
pub mod error;
pub mod event;
pub mod transaction;

use crate::data::{
    LIBRA_ADDRESS_SIZE, LIBRA_EVENT_KEY_SIZE, LIBRA_PRIVKEY_SIZE, LIBRA_PUBKEY_SIZE,
};
use libra_crypto::ed25519::{ED25519_PRIVATE_KEY_LENGTH, ED25519_PUBLIC_KEY_LENGTH};
use libra_types::{account_address::AccountAddress, event::EVENT_KEY_LENGTH};

static_assertions::const_assert_eq!(LIBRA_PUBKEY_SIZE, ED25519_PUBLIC_KEY_LENGTH as u32);
static_assertions::const_assert_eq!(LIBRA_PRIVKEY_SIZE, ED25519_PRIVATE_KEY_LENGTH as u32);
static_assertions::const_assert_eq!(LIBRA_ADDRESS_SIZE, AccountAddress::LENGTH as u32);
static_assertions::const_assert_eq!(LIBRA_EVENT_KEY_SIZE, EVENT_KEY_LENGTH as u32);
