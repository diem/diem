// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use anyhow::{ensure, Error, Result};
#[cfg(feature = "fuzzing")]
use proptest_derive::Arbitrary;
#[cfg(feature = "fuzzing")]
use rand::{rngs::OsRng, RngCore};
use serde::{de, ser, Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// A struct that represents a globally unique id for an Event stream that a user can listen to.
/// By design, the lower part of EventKey is the same as account address.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
pub struct EventKey([u8; EventKey::LENGTH]);

impl EventKey {
    /// Construct a new EventKey from a byte array slice.
    pub fn new(key: [u8; Self::LENGTH]) -> Self {
        EventKey(key)
    }

    /// The number of bytes in an EventKey.
    pub const LENGTH: usize = AccountAddress::LENGTH + 8;

    /// Get the byte representation of the event key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert event key into a byte array.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Get the account address part in this event key
    pub fn get_creator_address(&self) -> AccountAddress {
        let mut arr_bytes = [0u8; AccountAddress::LENGTH];
        arr_bytes.copy_from_slice(&self.0[EventKey::LENGTH - AccountAddress::LENGTH..]);

        AccountAddress::new(arr_bytes)
    }

    #[cfg(feature = "fuzzing")]
    /// Create a random event key for testing
    pub fn random() -> Self {
        let mut rng = OsRng::new().expect("can't access OsRng");
        let salt = rng.next_u64();
        EventKey::new_from_address(&AccountAddress::random(), salt)
    }

    /// Create a unique handle by using an AccountAddress and a counter.
    pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
        let mut output_bytes = [0; Self::LENGTH];
        let (lhs, rhs) = output_bytes.split_at_mut(8);
        lhs.copy_from_slice(&salt.to_le_bytes());
        rhs.copy_from_slice(addr.as_ref());
        EventKey(output_bytes)
    }
}

impl From<EventKey> for [u8; EventKey::LENGTH] {
    fn from(event_key: EventKey) -> Self {
        event_key.0
    }
}

impl From<&EventKey> for [u8; EventKey::LENGTH] {
    fn from(event_key: &EventKey) -> Self {
        event_key.0
    }
}

// TODO(#1307)
impl ser::Serialize for EventKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        // In order to preserve the Serde data model and help analysis tools,
        // make sure to wrap our value in a container with the same name
        // as the original type.
        serializer.serialize_newtype_struct("EventKey", &self.0[..])
    }
}

impl<'de> de::Deserialize<'de> for EventKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        // See comment in serialize.
        #[derive(::serde::Deserialize)]
        #[serde(rename = "EventKey")]
        struct Value<'a>(&'a [u8]);

        let value = Value::deserialize(deserializer)?;
        Self::try_from(value.0).map_err(<D::Error as ::serde::de::Error>::custom)
    }
}

impl TryFrom<&[u8]> for EventKey {
    type Error = Error;

    /// Tries to convert the provided byte array into Event Key.
    fn try_from(bytes: &[u8]) -> Result<EventKey> {
        ensure!(
            bytes.len() == Self::LENGTH,
            "The Event Key {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; Self::LENGTH];
        addr.copy_from_slice(bytes);
        Ok(EventKey(addr))
    }
}

/// A Rust representation of an Event Handle Resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventHandle {
    /// Number of events in the event stream.
    count: u64,
    /// The associated globally unique key that is used as the key to the EventStore.
    key: EventKey,
}

impl EventHandle {
    /// Constructs a new Event Handle
    pub fn new(key: EventKey, count: u64) -> Self {
        EventHandle { key, count }
    }

    /// Return the key to where this event is stored in EventStore.
    pub fn key(&self) -> &EventKey {
        &self.key
    }
    /// Return the counter for the handle
    pub fn count(&self) -> u64 {
        self.count
    }

    #[cfg(feature = "fuzzing")]
    pub fn count_mut(&mut self) -> &mut u64 {
        &mut self.count
    }

    #[cfg(feature = "fuzzing")]
    /// Create a random event handle for testing
    pub fn random_handle(count: u64) -> Self {
        Self {
            key: EventKey::random(),
            count,
        }
    }

    #[cfg(feature = "fuzzing")]
    /// Derive a unique handle by using an AccountAddress and a counter.
    pub fn new_from_address(addr: &AccountAddress, salt: u64) -> Self {
        Self {
            key: EventKey::new_from_address(addr, salt),
            count: 0,
        }
    }
}

impl fmt::LowerHex for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_bytes()))
    }
}

impl fmt::Display for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        // Forward to the LowerHex impl with a "0x" prepended (the # flag).
        write!(f, "{:#x}", self)
    }
}
