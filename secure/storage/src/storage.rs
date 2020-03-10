// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoStorage, KVStorage};

/// This is the Libra interface into secure storage. Any storage engine implementing this trait
/// should support both key/value operations (e.g., get, set and create) and cryptographic key
/// operations (e.g., generate_key, sign_message and rotate_key).
pub trait Storage: KVStorage + CryptoStorage {}

impl<T> Storage for T where T: KVStorage + CryptoStorage {}
