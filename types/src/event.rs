// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use anyhow::{ensure, Error, Result};
#[cfg(feature = "fuzzing")]
use rand::{rngs::OsRng, RngCore};
use serde::{de, ser, Deserialize, Serialize};
use std::{
    cmp::{Ord, Ordering, PartialOrd},
    convert::TryFrom,
    fmt,
    hash::Hash,
};

/// Size of an event key.
pub const EVENT_KEY_LENGTH: usize = 40;

/// A struct that represents a globally unique id for an Event stream that a user can listen to.
pub struct EventKey([u8; EVENT_KEY_LENGTH]);

impl EventKey {
    /// Construct a new EventKey from a byte array slice.
    pub fn new(key: [u8; EVENT_KEY_LENGTH]) -> Self {
        EventKey(key)
    }

    /// Get the byte representation of the event key.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert event key into a byte array.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
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
        let mut output_bytes = [0; EVENT_KEY_LENGTH];
        let (lhs, rhs) = output_bytes.split_at_mut(8);
        lhs.copy_from_slice(&salt.to_le_bytes());
        rhs.copy_from_slice(addr.as_ref());
        EventKey(output_bytes)
    }
}

// Rust doesn't support deriving traits for array size greater than 32. We'll implement those traits
// by hand for now
impl PartialEq for EventKey {
    #[inline]
    fn eq(&self, other: &EventKey) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for EventKey {}

impl PartialOrd for EventKey {
    fn partial_cmp(&self, other: &EventKey) -> Option<Ordering> {
        PartialOrd::partial_cmp(self.as_bytes(), other.as_bytes())
    }
    #[inline]
    fn lt(&self, other: &EventKey) -> bool {
        PartialOrd::lt(self.as_bytes(), other.as_bytes())
    }
    #[inline]
    fn le(&self, other: &EventKey) -> bool {
        PartialOrd::le(self.as_bytes(), other.as_bytes())
    }
    #[inline]
    fn ge(&self, other: &EventKey) -> bool {
        PartialOrd::ge(self.as_bytes(), other.as_bytes())
    }
    #[inline]
    fn gt(&self, other: &EventKey) -> bool {
        PartialOrd::gt(self.as_bytes(), other.as_bytes())
    }
}

impl Ord for EventKey {
    #[inline]
    fn cmp(&self, other: &EventKey) -> Ordering {
        Ord::cmp(self.as_bytes(), other.as_bytes())
    }
}

impl Clone for EventKey {
    fn clone(&self) -> Self {
        let mut key = [0; EVENT_KEY_LENGTH];
        key.copy_from_slice(self.as_bytes());
        Self(key)
    }
}

impl fmt::Debug for EventKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventKey({:?})", self.as_bytes())
    }
}

impl Copy for EventKey {}

impl Hash for EventKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Hash::hash(self.as_bytes(), state)
    }
}
// TODO(#1307)
impl ser::Serialize for EventKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> de::Deserialize<'de> for EventKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct EventKeyVisitor;

        impl<'de> de::Visitor<'de> for EventKeyVisitor {
            type Value = EventKey;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("EventKey in bytes")
            }

            fn visit_bytes<E>(self, value: &[u8]) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                EventKey::try_from(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_bytes(EventKeyVisitor)
    }
}

impl TryFrom<&[u8]> for EventKey {
    type Error = Error;

    /// Tries to convert the provided byte array into Event Key.
    fn try_from(bytes: &[u8]) -> Result<EventKey> {
        ensure!(
            bytes.len() == EVENT_KEY_LENGTH,
            "The Address {:?} is of invalid length",
            bytes
        );
        let mut addr = [0u8; EVENT_KEY_LENGTH];
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
